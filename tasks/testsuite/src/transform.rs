use std::{
    collections::BTreeMap,
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use rayon::prelude::*;

use crate::{
    cases::transform::TransformCase,
    runner::{Runner, babel_transform::output_root},
    suite::TestResult,
    util::crate_root,
};

const DIFFERENCE_REPORT_FILE: &str = "differences.md";

pub fn run<R: Runner<C>, C: TransformCase>() -> ExitCode {
    let cases = C::read();
    assert!(!cases.is_empty(), "no {} transform fixtures found", C::NAME);
    let output_root = output_root(R::NAME);
    if output_root.exists() {
        fs::remove_dir_all(&output_root).unwrap();
    }
    let (runnable, skipped): (Vec<_>, Vec<_>) = cases.into_iter().partition(R::should_run);
    let results = runnable
        .par_iter()
        .map(|case| (case.package_name().to_owned(), R::run(case)))
        .collect::<Vec<_>>();
    let suite_name = R::NAME.replace('_', " ");
    let snapshot = SnapshotReport::new(&results, &skipped).render(&suite_name);
    let snapshot_path = crate_root()
        .join("snapshots")
        .join(format!("{}.snap", R::NAME));

    fs::write(&snapshot_path, snapshot).unwrap();
    println!("Updated {}", snapshot_path.display());

    let difference_report =
        DifferenceReport::render(&suite_name, &runnable, &results, &output_root).unwrap();
    let difference_report_path = output_root.join(DIFFERENCE_REPORT_FILE);
    fs::create_dir_all(&output_root).unwrap();
    fs::write(&difference_report_path, difference_report).unwrap();

    println!("Wrote comparison outputs to {}", output_root.display());
    println!(
        "Wrote difference summary to {}",
        difference_report_path.display()
    );
    println!(
        "Compare with: diff -ru {} {}",
        output_root.join("expected").display(),
        output_root.join("actual").display()
    );

    if results
        .iter()
        .any(|(_, result)| matches!(result, TestResult::Panic { .. }))
    {
        eprintln!("Transform run contained one or more panics");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

struct DifferenceReport;

impl DifferenceReport {
    fn render<C: TransformCase>(
        suite_name: &str,
        cases: &[C],
        results: &[(String, TestResult)],
        output_root: &Path,
    ) -> io::Result<String> {
        let mut differences = String::new();
        let mut other_failures = String::new();
        let mut difference_count = 0;
        let mut other_failure_count = 0;

        for (case, (_, result)) in cases.iter().zip(results) {
            match result {
                TestResult::Failed { path, error, .. } => {
                    let expected_path = comparison_output_path(output_root, "expected", case);
                    let actual_path = comparison_output_path(output_root, "actual", case);
                    if expected_path.is_file() && actual_path.is_file() {
                        difference_count += 1;
                        let expected = fs::read_to_string(expected_path)?;
                        let actual = fs::read_to_string(actual_path)?;
                        writeln!(differences, "## {}\n", path.display()).unwrap();
                        writeln!(differences, "### Expected\n").unwrap();
                        write_indented(&mut differences, &expected);
                        writeln!(differences, "\n### Actual\n").unwrap();
                        write_indented(&mut differences, &actual);
                        differences.push('\n');
                    } else {
                        other_failure_count += 1;
                        writeln!(other_failures, "- `{}`: {error}", path.display()).unwrap();
                    }
                }
                TestResult::Panic { path, .. } => {
                    other_failure_count += 1;
                    writeln!(other_failures, "- `{}`: panicked", path.display()).unwrap();
                }
                TestResult::Passed { .. } => {}
            }
        }

        let mut report = format!(
            "# {suite_name} differences\n\nOutput differences: {difference_count}\n\nOther failures: {other_failure_count}\n"
        );
        if !other_failures.is_empty() {
            report.push_str("\n## Other failures\n\n");
            report.push_str(&other_failures);
        }
        if !differences.is_empty() {
            report.push('\n');
            report.push_str(&differences);
        }
        Ok(report)
    }
}

fn comparison_output_path<C: TransformCase>(output_root: &Path, kind: &str, case: &C) -> PathBuf {
    output_root.join(kind).join(
        case.output_path()
            .strip_prefix(crate_root())
            .expect("transform output path should be inside the testsuite root"),
    )
}

fn write_indented(output: &mut String, source: &str) {
    for line in source.split('\n') {
        writeln!(output, "    {line}").unwrap();
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
    fn new<C: TransformCase>(results: &[(String, TestResult)], skipped: &[C]) -> Self {
        let mut report = Self {
            total: Summary::default(),
            packages: BTreeMap::new(),
        };

        for case in skipped {
            report.total.skipped += 1;
            report
                .packages
                .entry(case.package_name().to_owned())
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

    fn render(self, suite_name: &str) -> String {
        let mut output = format!(
            "{suite_name}\n{}\n\nTotal\n-----\n",
            "=".repeat(suite_name.len())
        );
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
