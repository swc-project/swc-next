use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
};

use rayon::prelude::*;
use swc_next_allocator::Allocator;
use swc_next_ecma_ast::{Ast, Program, Severity};
use swc_next_ecma_codegen::{Options as CodegenOptions, print};
use swc_next_ecma_parser::{NoTokenParserConfig, Options as ParserOptions, Parser};
use swc_next_ecma_semantic::analyze;
use swc_next_ecma_transformer::{Transformer, TransformerOptions};
use testsuite::{cases::babel::BabelTransformCase, suite::TestResult, util::crate_root};

const SNAPSHOT_FILE: &str = "babel_transform.snap";

fn main() {
    let cases = BabelTransformCase::read();
    let (runnable, skipped): (Vec<_>, Vec<_>) = cases.into_iter().partition(is_runnable);
    let results = runnable
        .par_iter()
        .map(run_snapshot_case)
        .collect::<Vec<_>>();
    let snapshot = SnapshotReport::new(&results, &skipped).render();
    let snapshot_path = crate_root().join("snapshots").join(SNAPSHOT_FILE);

    fs::write(&snapshot_path, snapshot).unwrap();
    println!("Updated {}", snapshot_path.display());
}

fn is_runnable(case: &BabelTransformCase) -> bool {
    case.options.plugins.errors.is_empty()
        && case.options.plugins.unsupported.is_empty()
        && case.options.presets.errors.is_empty()
        && case.options.presets.unsupported.is_empty()
}

fn run_snapshot_case(case: &BabelTransformCase) -> (String, TestResult) {
    let path = case
        .path
        .strip_prefix(crate_root())
        .unwrap_or(&case.path)
        .to_path_buf();
    let result = match catch_unwind(AssertUnwindSafe(|| {
        let actual = transform_input(case)?;
        let expected = normalize_expected_output(case)?;
        if actual != expected {
            Err(format!(
                "{}: transformed output differs from output.js",
                case.name,
            ))
        } else {
            Ok(())
        }
    })) {
        Ok(Ok(())) => TestResult::Passed {
            path,
            negative: false,
        },
        Ok(Err(error)) => TestResult::Failed {
            path,
            error,
            negative: false,
        },
        Err(_) => TestResult::Panic {
            path,
            negative: false,
        },
    };

    (case.package_name.clone(), result)
}

// Snapshot rendering

#[derive(Default)]
struct Summary {
    passed: usize,
    failed: usize,
    panicked: usize,
    skipped: usize,
}

impl Summary {
    fn record_result(&mut self, result: &TestResult) {
        match result {
            TestResult::Passed { .. } => self.passed += 1,
            TestResult::Failed { .. } => self.failed += 1,
            TestResult::Panic { .. } => self.panicked += 1,
        }
    }

    fn total(&self) -> usize {
        self.passed + self.failed + self.panicked + self.skipped
    }

    fn write_to(&self, output: &mut String) {
        let total = self.total();
        let rate = |count| {
            if total == 0 {
                0.0
            } else {
                count as f64 / total as f64 * 100.0
            }
        };

        writeln!(output, "Cases: {total}").unwrap();
        writeln!(
            output,
            "Passed: {}/{} ({:.2}%)",
            self.passed,
            total,
            rate(self.passed)
        )
        .unwrap();
        writeln!(
            output,
            "Failed: {}/{} ({:.2}%)",
            self.failed,
            total,
            rate(self.failed)
        )
        .unwrap();
        writeln!(
            output,
            "Panicked: {}/{} ({:.2}%)",
            self.panicked,
            total,
            rate(self.panicked)
        )
        .unwrap();
        writeln!(
            output,
            "Skipped: {}/{} ({:.2}%)",
            self.skipped,
            total,
            rate(self.skipped)
        )
        .unwrap();
    }
}

#[derive(Default)]
struct PackageReport {
    summary: Summary,
    messages: Vec<String>,
}

impl PackageReport {
    fn record_result(&mut self, result: &TestResult) {
        self.summary.record_result(result);
        match result {
            TestResult::Passed { .. } => {}
            TestResult::Failed { path, error, .. } => self
                .messages
                .push(format!("Failed: {} - {error}", path.to_string_lossy())),
            TestResult::Panic { path, .. } => self
                .messages
                .push(format!("Panic: {}", path.to_string_lossy())),
        }
    }
}

struct SnapshotReport {
    total: Summary,
    packages: BTreeMap<String, PackageReport>,
}

impl SnapshotReport {
    fn new(results: &[(String, TestResult)], skipped: &[BabelTransformCase]) -> Self {
        let mut report = Self {
            total: Summary::default(),
            packages: BTreeMap::new(),
        };

        for case in skipped {
            report.total.skipped += 1;
            report
                .packages
                .entry(case.package_name.clone())
                .or_default()
                .summary
                .skipped += 1;
        }
        for (package_name, result) in results {
            report.total.record_result(result);
            report
                .packages
                .entry(package_name.clone())
                .or_default()
                .record_result(result);
        }

        report
    }

    fn render(self) -> String {
        let mut output = String::from("babel transform\n===============\n\nTotal\n-----\n");
        self.total.write_to(&mut output);
        output.push_str("\nPackages\n========\n");

        for (package_name, mut package) in self.packages {
            package.messages.sort_unstable();
            writeln!(output, "\n{package_name}").unwrap();
            writeln!(output, "{}", "-".repeat(package_name.len())).unwrap();
            package.summary.write_to(&mut output);
            if !package.messages.is_empty() {
                output.push_str("Messages:\n");
                for message in package.messages {
                    writeln!(output, "{message}").unwrap();
                }
            }
        }

        output
    }
}

fn transform_input(case: &BabelTransformCase) -> Result<Vec<u8>, String> {
    let source = case.input.as_bytes();
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = parse_program(&mut ast, case)?;
    let semantic = analyze(program, &mut ast);
    if let Some(diagnostic) = semantic
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format!(
            "semantic: {} at {}..{}",
            diagnostic.message, diagnostic.span.start, diagnostic.span.end
        ));
    }

    let options: TransformerOptions = case.options.clone().into();
    Transformer::transform(program, &mut ast, &semantic, options);
    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_vec())
}

fn normalize_expected_output(case: &BabelTransformCase) -> Result<Vec<u8>, String> {
    let source = case.output.as_bytes();
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = parse_program(&mut ast, case)?;
    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_vec())
}

fn parse_program(ast: &mut Ast<'_>, case: &BabelTransformCase) -> Result<Program, String> {
    let options = ParserOptions {
        source_type: case.source_type(),
        lang: case.lang(),
        ..ParserOptions::default()
    };
    let (program, error) = {
        let result = Parser::init(ast, options, NoTokenParserConfig).parse();
        let error = result
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| {
                format!(
                    "parse: {} at {}..{}",
                    diagnostic.message, diagnostic.span.start, diagnostic.span.end
                )
            });
        (result.program, error)
    };

    if let Some(error) = error {
        return Err(error);
    }
    Ok(program)
}
