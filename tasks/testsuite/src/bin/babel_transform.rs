use std::{collections::BTreeMap, fmt::Write as _, fs};

use rayon::prelude::*;
use testsuite::{
    cases::babel::BabelTransformCase,
    runner::{Runner, babel_transform::BabelTransformRunner},
    suite::TestResult,
    util::crate_root,
};

const SNAPSHOT_FILE: &str = "babel_transform.snap";

fn main() {
    let cases = BabelTransformCase::read();
    let (runnable, skipped): (Vec<_>, Vec<_>) = cases
        .into_iter()
        .partition(BabelTransformRunner::should_run);
    let results = runnable
        .par_iter()
        .map(|case| (case.package_name.clone(), BabelTransformRunner::run(case)))
        .collect::<Vec<_>>();
    let snapshot = SnapshotReport::new(&results, &skipped).render();
    let snapshot_path = crate_root().join("snapshots").join(SNAPSHOT_FILE);

    fs::write(&snapshot_path, snapshot).unwrap();
    println!("Updated {}", snapshot_path.display());
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
