use std::{collections::HashSet, fs};

use colored::*;
use pico_args::Arguments;
use rayon::ThreadPoolBuilder;

use crate::{
    cases::{Case, test262},
    runner::test262::Test262Runner,
    suite::TestResult,
    util::crate_root,
};

pub(crate) mod cases;
pub(crate) mod runner;
pub(crate) mod suite;
pub(crate) mod util;

// TODO(flat-ast): Restore after the swc_core comparison layer is updated for
// the current typed AST and semantic API.
// pub(crate) mod conformance;
// pub(crate) mod legacy;

pub struct AppArgs {
    pub debug: bool,
    pub cases: Vec<String>,
    pub runners: HashSet<String>,
    pub test262: bool,
    pub update_snapshots: bool,
}

const TEST262_RUNNER: &str = "test262";

pub fn main() {
    // Initialize args
    let mut args = Arguments::from_env();
    let args = AppArgs {
        debug: args.contains("--debug"),
        test262: args.contains("--test262"),
        update_snapshots: args.contains("--update-snapshots"),
        cases: args.values_from_str("--cases").unwrap(),
        runners: args
            .values_from_str("--runners")
            .unwrap()
            .into_iter()
            .collect(),
    };

    if args.test262 || args.runners.is_empty() || args.runners.contains(TEST262_RUNNER) {
        return test_test262_snapshot(&args);
    }

    eprintln!("No enabled runner selected. Use --test262 or --runners test262.");
}

fn test_test262_snapshot(args: &AppArgs) {
    if args.debug {
        ThreadPoolBuilder::new()
            .num_threads(1)
            .build_global()
            .unwrap();
    }

    let cases = filter(args, test262::Test262Case::read());
    let results = Test262Runner::run(args, &cases);
    let snapshot = {
        let total = results.len();
        let mut passed = 0;
        let mut failed = 0;
        let mut panicked = 0;
        let mut diagnostics_vec = Vec::new();

        for result in &results {
            match result {
                TestResult::Passed { .. } => {
                    passed += 1;
                }
                TestResult::Failed { path, error } => {
                    failed += 1;
                    let path = path.display().to_string().replace('\\', "/");
                    diagnostics_vec.push(format!("Failed: {} - {}", path, error));
                }
                TestResult::Panic { path } => {
                    panicked += 1;
                    let path = path.display().to_string().replace('\\', "/");
                    diagnostics_vec.push(format!("Panic: {}", path));
                }
            }
        }

        diagnostics_vec.sort();

        let rate = |count| {
            if total == 0 {
                0.0
            } else {
                (count as f64 / total as f64) * 100.0
            }
        };
        let mut result = String::from("Test262 parser + semantic pipeline\n");
        result.push_str("==================================\n");
        result.push_str(&format!(
            "Passed: {passed}/{total} ({:.2}%)\n",
            rate(passed),
        ));
        result.push_str(&format!(
            "Failed: {failed}/{total} ({:.2}%)\n",
            rate(failed),
        ));
        result.push_str(&format!(
            "Panicked: {panicked}/{total} ({:.2}%)\n",
            rate(panicked),
        ));

        for diag in diagnostics_vec {
            result.push_str(&diag);
            result.push('\n');
        }

        result
    };

    let snapshot_path = crate_root().join("snapshots").join("test262.snap");
    if !args.cases.is_empty() {
        print!("{snapshot}");
        return;
    }
    let expected = fs::read_to_string(&snapshot_path).ok();
    if args.update_snapshots || expected.is_none() {
        fs::write(&snapshot_path, &snapshot).unwrap();
        println!("Updated {}", snapshot_path.display());
        return;
    }
    if expected.as_deref() != Some(&snapshot) {
        let diff = similar::TextDiff::from_lines(expected.as_deref().unwrap(), &snapshot);
        let mut unified = diff.unified_diff();
        unified.header("snapshot", "current");
        eprintln!("Test262 snapshot mismatch:\n{unified}");
        std::process::exit(1);
    }
    println!("{}", "Test262 snapshot matched".green());
}

fn filter<T: Case>(args: &AppArgs, list: Vec<T>) -> Vec<T> {
    if args.cases.is_empty() {
        return list;
    }

    list.into_iter()
        .filter(|case| {
            args.cases
                .iter()
                .any(|f| case.path().to_string_lossy().as_ref().contains(f))
        })
        .collect()
}
