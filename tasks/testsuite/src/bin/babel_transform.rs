use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs,
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
};

use colored::Colorize;
use pico_args::Arguments;
use rayon::prelude::*;
use swc_experimental_allocator::Allocator;
use swc_experimental_ecma_ast::{Ast, Lang, Program, Severity, SourceType};
use swc_experimental_ecma_codegen::{Options as CodegenOptions, print};
use swc_experimental_ecma_parser::{Options as ParserOptions, Parser};
use swc_experimental_ecma_semantic::analyze;
use swc_experimental_ecma_transformer::{Transformer, TransformerOptions};
use testsuite::{cases::babel::BabelCase, suite::TestResult, util::crate_root};

const SNAPSHOT_FILE: &str = "babel_transform.snap";
const DETAIL_SEPARATOR_WIDTH: usize = 80;

struct AppArgs {
    case_filter: Option<String>,
}

impl AppArgs {
    fn parse() -> Self {
        let mut args = Arguments::from_env();
        let case_filter = args.opt_value_from_str("--case").unwrap();
        assert!(
            !args.contains("--case"),
            "`--case` can only be specified once"
        );
        Self { case_filter }
    }
}

fn main() {
    let args = AppArgs::parse();
    let cases = BabelCase::read();

    match args.case_filter {
        Some(filter) => print_selected_case(cases, &filter),
        None => update_snapshot(cases),
    }
}

fn update_snapshot(cases: Vec<BabelCase>) {
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

fn is_runnable(case: &BabelCase) -> bool {
    case.options.plugins.errors.is_empty()
        && case.options.plugins.unsupported.is_empty()
        && case.options.presets.errors.is_empty()
        && case.options.presets.unsupported.is_empty()
}

fn unsupported_reason(case: &BabelCase) -> String {
    let entries = case
        .options
        .plugins
        .errors
        .iter()
        .chain(&case.options.plugins.unsupported)
        .chain(&case.options.presets.errors)
        .chain(&case.options.presets.unsupported)
        .cloned()
        .collect::<Vec<_>>();
    format!("unsupported options: {}", entries.join(", "))
}

// Snapshot execution

struct CaseExecution {
    package_name: String,
    result: TestResult,
}

fn run_snapshot_case(case: &BabelCase) -> CaseExecution {
    let path = case
        .path
        .strip_prefix(crate_root())
        .unwrap_or(&case.path)
        .to_path_buf();
    let result = match catch_unwind(AssertUnwindSafe(|| validate_case(case))) {
        Ok(Ok(())) => TestResult::Passed { path },
        Ok(Err(error)) => TestResult::Failed { path, error },
        Err(_) => TestResult::Panic { path },
    };

    CaseExecution {
        package_name: case.package_name.clone(),
        result,
    }
}

// Detailed case output

fn print_selected_case(cases: Vec<BabelCase>, filter: &str) {
    let mut matches = cases.into_iter().filter(|case| case.name.contains(filter));
    let Some(case) = matches.next() else {
        println!("No Babel transform case matched `{filter}`.");
        return;
    };

    if matches.next().is_some() {
        println!("Multiple Babel transform cases matched `{filter}`; use a more specific value.");
        return;
    }

    print_case_detail(&case);
}

fn print_case_detail(case: &BabelCase) {
    println!("{}", "=".repeat(DETAIL_SEPARATOR_WIDTH));
    println!("Case: {}", case.name);
    println!("Package: {}", case.package_name);
    println!("Input: {}", normalize_path(&case.path));

    if !is_runnable(case) {
        println!("Status: {}", "Unsupported".yellow());
        println!("Reason: {}", unsupported_reason(case));
        return;
    }

    match catch_unwind(AssertUnwindSafe(|| generate_artifacts(case))) {
        Ok(Ok(artifacts)) => print_artifact_comparison(&artifacts),
        Ok(Err(error)) => {
            println!("Status: {}", "Failed".red());
            println!("Message: {error}");
        }
        Err(_) => println!("Status: {}", "Panicked".red()),
    }
}

fn print_artifact_comparison(artifacts: &Artifacts) {
    if let Some(offset) = artifacts.first_difference() {
        println!("Status: {}", "Failed".red());
        println!(
            "First difference: byte {offset} (actual: {} bytes, expected: {} bytes)",
            artifacts.actual.len(),
            artifacts.expected.len()
        );
    } else {
        println!("Status: {}", "Passed".green());
        println!("Outputs are identical ({} bytes).", artifacts.actual.len());
    }

    print_artifact("Actual (input.js → transform → codegen)", &artifacts.actual);
    print_artifact("Expected (output.js → codegen)", &artifacts.expected);
}

fn print_artifact(label: &str, artifact: &[u8]) {
    println!("--- {label} ---");
    print!("{}", String::from_utf8_lossy(artifact));
    if !artifact.ends_with(b"\n") {
        println!();
    }
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
            TestResult::Failed { path, error } => self
                .messages
                .push(format!("Failed: {} - {error}", normalize_path(path))),
            TestResult::Panic { path } => self
                .messages
                .push(format!("Panic: {}", normalize_path(path))),
        }
    }
}

struct SnapshotReport {
    total: Summary,
    packages: BTreeMap<String, PackageReport>,
}

impl SnapshotReport {
    fn new(results: &[CaseExecution], skipped: &[BabelCase]) -> Self {
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
        for execution in results {
            report.total.record_result(&execution.result);
            report
                .packages
                .entry(execution.package_name.clone())
                .or_default()
                .record_result(&execution.result);
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

// Transform pipeline

struct Artifacts {
    actual: Vec<u8>,
    expected: Vec<u8>,
}

impl Artifacts {
    fn first_difference(&self) -> Option<usize> {
        self.actual
            .iter()
            .zip(&self.expected)
            .position(|(actual, expected)| actual != expected)
            .or_else(|| {
                (self.actual.len() != self.expected.len())
                    .then(|| self.actual.len().min(self.expected.len()))
            })
    }
}

fn validate_case(case: &BabelCase) -> Result<(), String> {
    let artifacts = generate_artifacts(case)?;
    let Some(offset) = artifacts.first_difference() else {
        return Ok(());
    };

    Err(format!(
        "{}: transformed output differs from output.js at byte {offset} (actual: {} bytes, expected: {} bytes)",
        case.name,
        artifacts.actual.len(),
        artifacts.expected.len(),
    ))
}

fn generate_artifacts(case: &BabelCase) -> Result<Artifacts, String> {
    let actual = transform_input(case)?;
    let expected = normalize_expected_output(case)?;
    Ok(Artifacts { actual, expected })
}

fn transform_input(case: &BabelCase) -> Result<Vec<u8>, String> {
    let source = case.input.as_bytes();
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = parse_program(
        &mut ast,
        source,
        case.source_type(),
        case.lang(),
        "input.js parse",
    )?;
    let semantic = analyze(program, &mut ast);
    if let Some(diagnostic) = semantic
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.severity == Severity::Error)
    {
        return Err(format_diagnostic("input.js semantic", diagnostic));
    }

    let options: TransformerOptions = case.options.clone().into();
    Transformer::transform(program, &mut ast, &semantic, options);
    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_vec())
}

fn normalize_expected_output(case: &BabelCase) -> Result<Vec<u8>, String> {
    let source = case.output.as_bytes();
    let allocator = Allocator::new();
    let mut ast = Ast::new(source, &allocator);
    let program = parse_program(
        &mut ast,
        source,
        case.source_type(),
        case.lang(),
        "output.js parse",
    )?;
    Ok(print(&allocator, &ast, program, CodegenOptions::default())
        .code
        .to_vec())
}

fn parse_program(
    ast: &mut Ast<'_>,
    source: &[u8],
    source_type: SourceType,
    lang: Lang,
    stage: &str,
) -> Result<Program, String> {
    let options = ParserOptions {
        source_type,
        lang,
        ..ParserOptions::default()
    };
    let (program, error) = {
        let mut parser = Parser::init(ast, source, options);
        let program = parser.parse();
        let error = parser
            .diagnostics
            .iter()
            .find(|diagnostic| diagnostic.severity == Severity::Error)
            .map(|diagnostic| format_diagnostic(stage, diagnostic));
        (program, error)
    };

    if let Some(error) = error {
        return Err(error);
    }
    program.ok_or_else(|| format!("{stage}: parsing stopped without producing an error diagnostic"))
}

fn format_diagnostic(
    stage: &str,
    diagnostic: &swc_experimental_ecma_ast::Diagnostic<'_>,
) -> String {
    format!(
        "{stage}: {} at {}..{}",
        diagnostic.message, diagnostic.span.start, diagnostic.span.end
    )
}

fn normalize_path(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}
