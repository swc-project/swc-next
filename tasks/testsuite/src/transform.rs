use std::{
    fmt::Write as _,
    fs, io,
    path::{Path, PathBuf},
    process::ExitCode,
};

use rayon::prelude::*;

use crate::{
    cases::TransformCase,
    runner::{Runner, transform::output_root},
    suite::{TestResult, to_snapshot},
    util::crate_root,
};

const DIFFERENCE_REPORT_FILE: &str = "differences.md";

pub fn run<R: Runner<C>, C: TransformCase>() -> ExitCode {
    let cases = C::read();
    assert!(!cases.is_empty(), "no {} transform fixtures found", C::NAME);
    let output_root = output_root(R::NAME).join(C::NAME);
    if output_root.exists() {
        fs::remove_dir_all(&output_root).unwrap();
    }
    let results = cases.par_iter().map(R::run).collect::<Vec<_>>();
    let suite_name = R::NAME.replace('_', " ");
    let snapshot = to_snapshot(R::NAME, C::NAME, &results);
    let snapshot_path =
        crate_root()
            .join("snapshots")
            .join(format!("{}_{}.snap", R::NAME, C::NAME));

    fs::write(&snapshot_path, snapshot).unwrap();
    println!("Updated {}", snapshot_path.display());

    let difference_report =
        DifferenceReport::render(&suite_name, &cases, &results, &output_root).unwrap();
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
        .any(|result| matches!(result, TestResult::Panic { .. }))
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
        results: &[TestResult],
        output_root: &Path,
    ) -> io::Result<String> {
        let mut differences = String::new();
        let mut other_failures = String::new();
        let mut difference_count = 0;
        let mut other_failure_count = 0;

        for (case, result) in cases.iter().zip(results) {
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
                TestResult::Passed { .. } | TestResult::Skipped { .. } => {}
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
