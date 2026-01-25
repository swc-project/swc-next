use std::{collections::HashSet, fs};

use colored::*;
use pico_args::Arguments;
use rayon::ThreadPoolBuilder;

use crate::{
    cases::{
        Case,
        misc::MiscCase,
        test262,
        test262_parser::{self},
    },
    runner::{
        parser::ParserRunner, semantic::SemanticRunner, transform_remove_paren::RemoveParenRunner,
    },
    suite::TestResult,
    util::crate_root,
};

mod cases;
mod runner;
mod suite;
mod util;

pub struct AppArgs {
    pub debug: bool,
    pub cases: Vec<String>,
    pub runners: HashSet<String>,
    pub failures: HashSet<String>,
    pub test262: bool,
}

const PARSER_RUNNER: &str = "parser";
const SEMANTIC_RUNNER: &str = "semantic";
const REMOVE_PAREN_RUNNER: &str = "remove_paren";

pub fn main() {
    // Initialize args
    let mut args = Arguments::from_env();
    let args = AppArgs {
        debug: args.contains("--debug"),
        test262: args.contains("--test262"),
        cases: args.values_from_str("--cases").unwrap(),
        runners: args
            .values_from_str("--runners")
            .unwrap()
            .into_iter()
            .collect(),
        failures: args
            .values_from_str("--failures")
            .unwrap()
            .into_iter()
            .collect(),
    };

    if args.test262 {
        return test_test262_snapshots(&args);
    }

    test_normal(&args);
}

fn test_normal(args: &AppArgs) {
    if args.debug {
        ThreadPoolBuilder::new()
            .num_threads(1)
            .build_global()
            .unwrap();
    }

    let mut results = Vec::new();

    let misc_cases = filter(args, MiscCase::read());
    let test262_parser_cases = filter(args, test262_parser::Test262ParserCase::read());

    if args.runners.is_empty() || args.runners.contains(PARSER_RUNNER) {
        results.extend(ParserRunner::run(args, &misc_cases));
        results.extend(ParserRunner::run(args, &test262_parser_cases));
    }

    if args.runners.is_empty() || args.runners.contains(SEMANTIC_RUNNER) {
        results.extend(SemanticRunner::run(args, &misc_cases));
        results.extend(SemanticRunner::run(args, &test262_parser_cases));
    }

    if args.runners.is_empty() || args.runners.contains(REMOVE_PAREN_RUNNER) {
        results.extend(RemoveParenRunner::run(args, &misc_cases));
        results.extend(RemoveParenRunner::run(args, &test262_parser_cases));
    }

    // Collect results
    let mut passed = 0;
    let mut failed = 0;
    let mut panicked = 0;
    let mut ignored = 0;
    for result in results {
        match result {
            TestResult::Passed { .. } => {
                passed += 1;
            }
            TestResult::Failed { path, error } => {
                failed += 1;
                if args.failures.is_empty() || args.failures.contains("failed") {
                    println!("Failed: {} - {}", path.display(), error.red());
                }
            }
            TestResult::Panic { path } => {
                panicked += 1;
                if args.failures.is_empty() || args.failures.contains("panic") {
                    println!("Panic: {}", path.display());
                }
            }
            TestResult::Ignored { .. } => {
                ignored += 1;
            }
        }
    }

    println!("\n{}", "Test Summary:".bold());
    println!("Passed: {}", passed.to_string().green());
    println!("Failed: {}", failed.to_string().red());
    println!("Ignored: {}", ignored.to_string().yellow());
    println!("Panicked: {}", panicked.to_string().yellow());

    if failed > 0 {
        std::process::exit(1);
    }
}

fn test_test262_snapshots(args: &AppArgs) {
    let cases = filter(args, test262::Test262Case::read());

    let to_snapshot = |results: &[TestResult]| {
        // Collect results
        let mut passed = 0;
        let mut failed = 0;
        let mut panicked = 0;
        let mut ignored = 0;
        let mut diagnostics_vec = Vec::new();

        for result in results {
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
                TestResult::Ignored { .. } => {
                    ignored += 1;
                }
            }
        }

        // 按字典顺序排序 diagnostics
        diagnostics_vec.sort();

        let total = results.len();
        let mut result = String::from("Test Summary: \n");
        result.push_str(&format!(
            "Passed: {passed}/{total} ({:.2}%)\n",
            (passed as f64 / total as f64) * 100.0,
        ));
        result.push_str(&format!(
            "Failed: {failed}/{total} ({:.2}%)\n",
            (failed as f64 / total as f64) * 100.0,
        ));
        result.push_str(&format!(
            "Panicked: {panicked}/{total} ({:.2}%)\n",
            (panicked as f64 / total as f64) * 100.0,
        ));
        result.push_str(&format!(
            "Ignored: {ignored}/{total} ({:.2}%)\n",
            (ignored as f64 / total as f64) * 100.0,
        ));

        for diag in diagnostics_vec {
            result.push_str(&diag);
            result.push('\n');
        }

        result
    };

    let snapshot_dir = crate_root().join("snapshots");
    if args.runners.is_empty() || args.runners.contains(PARSER_RUNNER) {
        let results = ParserRunner::run(args, &cases);
        fs::write(
            snapshot_dir.join("parser_test262.snap"),
            to_snapshot(&results),
        )
        .unwrap();
    }

    if args.runners.is_empty() || args.runners.contains(SEMANTIC_RUNNER) {
        let results = SemanticRunner::run(args, &cases);
        fs::write(
            snapshot_dir.join("semantic_test262.snap"),
            to_snapshot(&results),
        )
        .unwrap();
    }

    if args.runners.is_empty() || args.runners.contains(REMOVE_PAREN_RUNNER) {
        let results = RemoveParenRunner::run(args, &cases);
        fs::write(
            snapshot_dir.join("remove_paren_test262.snap"),
            to_snapshot(&results),
        )
        .unwrap();
    }
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
