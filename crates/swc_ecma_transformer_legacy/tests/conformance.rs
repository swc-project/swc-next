use std::{
    fmt::Write,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{AstCtx, Diagnostic, Severity};
use swc_next_ecma_codegen::generate;
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_transformer_legacy::Transformer;

#[path = "conformance/options.rs"]
mod options;

fn inputs(root: &Path) -> Vec<(PathBuf, PathBuf)> {
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
            if entry.file_type().is_file()
                && matches!(
                    path.extension().and_then(|v| v.to_str()),
                    Some("js" | "jsx" | "ts" | "tsx")
                )
            {
                let output = input
                    .parent()
                    .unwrap()
                    .join("output")
                    .join(path.strip_prefix(&input).unwrap());
                inputs.push((path, output));
            }
        }
    }
    inputs.sort_unstable();
    inputs
}

fn check_diagnostics(diagnostics: &[Diagnostic]) -> Result<(), String> {
    if let Some(diagnostic) = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(match &diagnostic.help {
            Some(help) => format!("{}: {help}", diagnostic.message),
            None => diagnostic.message.to_string(),
        });
    }
    Ok(())
}

fn parse<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: ParserOptions,
) -> Result<AstCtx<'a>, String> {
    let parsed = Parser::init(allocator, source, options, NoTokenParserConfig).parse();
    check_diagnostics(&parsed.diagnostics).map_err(|error| format!("parser: {error}"))?;
    Ok(parsed.ast)
}

fn run(input: &Path, output: &Path, root: &Path) -> Result<bool, String> {
    let Some(mut options) =
        options::read(input, root).map_err(|error| format!("configuration: {error}"))?
    else {
        return Ok(false);
    };
    // Compare canonical output, ignoring formatting and redundant parentheses.
    options.parser.preserve_parens = false;
    options.generator.minify = true;
    let raw = output.with_added_extension("raw");
    let output = if raw.is_file() { &raw } else { output };
    let source = fs::read_to_string(input).map_err(|error| error.to_string())?;
    let expected = fs::read_to_string(output).map_err(|error| format!("snapshot: {error}"))?;
    let allocator = Allocator::new();
    let mut ast = parse(&allocator, &source, options.parser)?;
    let result = Transformer::transform(&mut ast, options.transformer);
    check_diagnostics(&result.diagnostics).map_err(|error| format!("transform: {error}"))?;
    let expected_allocator = Allocator::new();
    let expected = parse(&expected_allocator, &expected, options.parser)
        .map_err(|error| format!("snapshot {error}"))?;
    // Generated purity flags and source annotations must pass through the same
    // parser/printer path before comparing their formatting.
    let actual = generate(&ast, options.generator.clone()).code;
    let actual_allocator = Allocator::new();
    let actual = parse(&actual_allocator, &actual, options.parser)
        .map_err(|error| format!("output {error}"))?;
    if generate(&actual, options.generator.clone()).code
        != generate(&expected, options.generator).code
    {
        return Err("transformed output differs from SWC snapshot".into());
    }
    Ok(true)
}

#[test]
#[ignore = "requires pinned SWC fixtures; run with --ignored"]
fn swc() {
    std::thread::Builder::new()
        .stack_size(32 * 1024 * 1024)
        .spawn(run_suite)
        .unwrap()
        .join()
        .unwrap();
}

fn run_suite() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = crate_root.join("../../tasks/testsuite/fixtures/swc/crates/swc/tests/compile");
    let cases = inputs(&root);
    let mut passed = 0;
    let mut skipped = 0;
    let mut failed = Vec::new();
    let mut panics = Vec::new();
    for (input, output) in &cases {
        let path = input
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        match catch_unwind(AssertUnwindSafe(|| run(input, output, &root))) {
            Ok(Ok(true)) => passed += 1,
            Ok(Ok(false)) => skipped += 1,
            Ok(Err(error)) => failed.push(format!("Failed: {path} - {error}")),
            Err(_) => panics.push(format!("Panic: {path}")),
        }
    }
    let mut snapshot = format!(
        "transform_legacy * swc\n========================\nTotal: {}\nPassed: {passed}\nFailed: {}\nPanicked: {}\nSkipped: {skipped}\n",
        cases.len(),
        failed.len(),
        panics.len(),
    );
    for line in failed.iter().chain(&panics) {
        writeln!(snapshot, "{line}").unwrap();
    }
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(crate_root.join("tests/snapshots"));
    settings.set_prepend_module_to_snapshot(false);
    settings.set_omit_expression(true);
    settings.bind(|| insta::assert_snapshot!("conformance_swc", snapshot));
    assert!(panics.is_empty(), "SWC conformance contained panics");
}
