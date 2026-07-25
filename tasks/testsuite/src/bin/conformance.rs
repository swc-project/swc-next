use std::{collections::HashSet, fs, process::ExitCode};

use pico_args::Arguments;
use rayon::ThreadPoolBuilder;
use testsuite::{
    cases::{Case, babel::BabelCase, test262::Test262Case, typescript::TypeScriptCase},
    runner::{Runner, codegen::CodegenRunner, parser::ParserRunner},
    suite::TestResult,
    util::crate_root,
};

pub struct AppArgs {
    pub debug: bool,
    pub cases: Vec<String>,
    pub runners: HashSet<String>,
}

pub fn main() -> ExitCode {
    // Initialize args
    let mut args = Arguments::from_env();
    let args = AppArgs {
        debug: args.contains("--debug"),
        cases: args.values_from_str("--cases").unwrap(),
        runners: args
            .values_from_str("--runners")
            .unwrap()
            .into_iter()
            .collect(),
    };

    if args.debug {
        ThreadPoolBuilder::new()
            .num_threads(1)
            .build_global()
            .unwrap();
    }

    let panicked = [
        update_snapshot::<ParserRunner, Test262Case>(&args),
        update_snapshot::<ParserRunner, TypeScriptCase>(&args),
        update_snapshot::<ParserRunner, BabelCase>(&args),
        update_snapshot::<CodegenRunner, Test262Case>(&args),
        update_snapshot::<CodegenRunner, TypeScriptCase>(&args),
        update_snapshot::<CodegenRunner, BabelCase>(&args),
    ]
    .into_iter()
    .any(|panicked| panicked);

    if panicked {
        eprintln!("Conformance run contained one or more panics");
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}

fn update_snapshot<R: Runner, C: Case>(args: &AppArgs) -> bool {
    if !args.runners.is_empty() && !args.runners.contains(R::NAME) {
        return false;
    }

    let cases = filter(args, C::read());
    let results = R::run(&cases, args.debug);
    let panicked = results
        .iter()
        .any(|result| matches!(result, TestResult::Panic { .. }));
    let snapshot = to_snapshot(R::NAME, C::NAME, &results);
    let snapshot_path =
        crate_root()
            .join("snapshots")
            .join(format!("{}_{}.snap", R::NAME, C::NAME));
    fs::write(&snapshot_path, snapshot).unwrap();
    println!("Updated {}", snapshot_path.display());
    panicked
}

fn to_snapshot(runner: &str, cases: &str, results: &[TestResult]) -> String {
    let total = results.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut panicked = 0;
    let mut diagnostics = Vec::new();

    for result in results {
        match result {
            TestResult::Passed { .. } => passed += 1,
            TestResult::Failed { path, error } => {
                failed += 1;
                let path = path.display().to_string().replace('\\', "/");
                diagnostics.push(format!("Failed: {path} - {error}"));
            }
            TestResult::Panic { path } => {
                panicked += 1;
                let path = path.display().to_string().replace('\\', "/");
                diagnostics.push(format!("Panic: {path}"));
            }
        }
    }

    diagnostics.sort();
    let rate = |count| {
        if total == 0 {
            0.0
        } else {
            (count as f64 / total as f64) * 100.0
        }
    };
    let mut snapshot = format!("{runner} * {cases}\n========================\n");
    snapshot.push_str(&format!(
        "Passed: {passed}/{total} ({:.2}%)\n",
        rate(passed),
    ));
    snapshot.push_str(&format!(
        "Failed: {failed}/{total} ({:.2}%)\n",
        rate(failed),
    ));
    snapshot.push_str(&format!(
        "Panicked: {panicked}/{total} ({:.2}%)\n",
        rate(panicked),
    ));
    for diagnostic in diagnostics {
        snapshot.push_str(&diagnostic);
        snapshot.push('\n');
    }
    snapshot
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
