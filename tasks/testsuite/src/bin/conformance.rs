use std::{fs, process::ExitCode};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use testsuite::{
    cases::{
        Case, babel_parser::BabelParserCase, swc_resolver::SwcResolverCase, test262::Test262Case,
        typescript::TypeScriptCase,
    },
    runner::{Runner, codegen::CodegenRunner, parser::ParserRunner, resolver::ResolverRunner},
    suite::TestResult,
    util::crate_root,
};

pub fn main() -> ExitCode {
    let panicked = [
        update_snapshots::<Test262Case>(),
        update_snapshots::<TypeScriptCase>(),
        update_snapshots::<BabelParserCase>(),
        update_snapshot::<ResolverRunner, SwcResolverCase>(&SwcResolverCase::read()),
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

fn update_snapshots<C: Case>() -> bool {
    let cases = C::read();
    [
        update_snapshot::<ParserRunner, C>(&cases),
        update_snapshot::<CodegenRunner, C>(&cases),
    ]
    .into_iter()
    .any(|panicked| panicked)
}

fn update_snapshot<R: Runner<C>, C: Case>(cases: &[C]) -> bool {
    let results = cases
        .par_iter()
        .filter(|case| R::should_run(*case))
        .map(R::run)
        .collect::<Vec<_>>();
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
    let mut positive = 0;
    let mut negative = 0;
    let mut positive_passed = 0;
    let mut negative_passed = 0;
    let mut panicked = 0;
    let mut diagnostics = Vec::new();

    for result in results {
        let is_negative = match result {
            TestResult::Passed { negative, .. }
            | TestResult::Failed { negative, .. }
            | TestResult::Panic { negative, .. } => *negative,
        };
        if is_negative {
            negative += 1;
        } else {
            positive += 1;
        }

        match result {
            TestResult::Passed {
                negative: false, ..
            } => positive_passed += 1,
            TestResult::Passed { negative: true, .. } => negative_passed += 1,
            TestResult::Failed { path, error, .. } => {
                let path = path.display().to_string().replace('\\', "/");
                diagnostics.push(format!("Failed: {path} - {error}"));
            }
            TestResult::Panic { path, .. } => {
                panicked += 1;
                let path = path.display().to_string().replace('\\', "/");
                diagnostics.push(format!("Panic: {path}"));
            }
        }
    }

    diagnostics.sort();
    let rate = |count, denominator| {
        if denominator == 0 {
            0.0
        } else {
            (count as f64 / denominator as f64) * 100.0
        }
    };
    let mut snapshot = format!("{runner} * {cases}\n========================\n");
    snapshot.push_str(&format!(
        "Positive Passed: {positive_passed}/{positive} ({:.2}%)\n",
        rate(positive_passed, positive),
    ));
    snapshot.push_str(&format!(
        "Negative Passed: {negative_passed}/{negative} ({:.2}%)\n",
        rate(negative_passed, negative),
    ));
    snapshot.push_str(&format!(
        "Panicked: {panicked}/{total} ({:.2}%)\n",
        rate(panicked, total),
    ));
    for diagnostic in diagnostics {
        snapshot.push_str(&diagnostic);
        snapshot.push('\n');
    }
    snapshot
}
