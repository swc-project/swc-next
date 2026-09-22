use std::{fs, io, process::ExitCode};

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use testsuite::{
    cases::{
        Case, babel_parser::BabelParserCase, swc_resolver::SwcResolverCase, test262::Test262Case,
        typescript::TypeScriptCase,
    },
    runner::{
        Runner, ast_round_trip::AstRoundTripRunner, codegen::CodegenRunner, parser::ParserRunner,
        resolver::ResolverRunner,
    },
    suite::TestResult,
    util::crate_root,
};

pub fn main() -> io::Result<ExitCode> {
    let panicked = [
        update_snapshots::<Test262Case>()?,
        update_snapshots::<TypeScriptCase>()?,
        update_snapshots::<BabelParserCase>()?,
        update_snapshot::<ResolverRunner, SwcResolverCase>(&SwcResolverCase::read())?,
    ]
    .into_iter()
    .any(|panicked| panicked);

    if panicked {
        eprintln!("Conformance run contained one or more panics");
        Ok(ExitCode::FAILURE)
    } else {
        Ok(ExitCode::SUCCESS)
    }
}

fn update_snapshots<C: Case>() -> io::Result<bool> {
    let cases = C::read();
    let panicked = [
        update_snapshot::<ParserRunner, C>(&cases)?,
        update_codegen_snapshot(&cases)?,
    ]
    .into_iter()
    .any(|panicked| panicked);
    Ok(panicked)
}

fn update_snapshot<R: Runner<C>, C: Case>(cases: &[C]) -> io::Result<bool> {
    let results = run_cases::<R, C>(cases);
    let snapshot = to_snapshot(R::NAME, C::NAME, &results);
    write_snapshot(R::NAME, C::NAME, snapshot)?;
    Ok(has_panics(&results))
}

fn run_cases<R: Runner<C>, C: Case>(cases: &[C]) -> Vec<TestResult> {
    cases
        .par_iter()
        .filter(|case| R::should_run(*case))
        .map(R::run)
        .collect()
}

fn has_panics(results: &[TestResult]) -> bool {
    results
        .iter()
        .any(|result| matches!(result, TestResult::Panic { .. }))
}

fn update_codegen_snapshot<C: Case>(cases: &[C]) -> io::Result<bool> {
    let codegen = run_cases::<CodegenRunner, C>(cases);
    let without_parens = run_cases::<AstRoundTripRunner<false>, C>(cases);
    let with_parens = run_cases::<AstRoundTripRunner<true>, C>(cases);
    let mut snapshot = to_snapshot("codegen", C::NAME, &codegen);
    snapshot.push_str(&to_round_trip_snapshot(&without_parens, false));
    snapshot.push_str(&to_round_trip_snapshot(&with_parens, true));
    write_snapshot("codegen", C::NAME, snapshot)?;
    Ok(has_panics(&codegen) || has_panics(&without_parens) || has_panics(&with_parens))
}

fn write_snapshot(runner: &str, cases: &str, snapshot: String) -> io::Result<()> {
    let snapshot_path = crate_root()
        .join("snapshots")
        .join(format!("{runner}_{cases}.snap"));
    fs::write(&snapshot_path, snapshot)?;
    println!("Updated {}", snapshot_path.display());
    Ok(())
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
            | TestResult::Panic { negative, .. }
            | TestResult::Skipped { negative, .. } => *negative,
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
            TestResult::Skipped { path, reason, .. } => {
                let path = path.display().to_string().replace('\\', "/");
                diagnostics.push(format!("Skipped: {path} - {reason}"));
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

fn to_round_trip_snapshot(results: &[TestResult], preserve_parens: bool) -> String {
    let total = results.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut panicked = 0;
    let mut diagnostics = Vec::new();
    for result in results {
        let (status, path, error) = match result {
            TestResult::Passed { .. } => {
                passed += 1;
                continue;
            }
            TestResult::Failed { path, error, .. } => {
                failed += 1;
                ("Failed", path, error.as_str())
            }
            TestResult::Skipped { path, reason, .. } => {
                skipped += 1;
                ("Skipped", path, reason.as_str())
            }
            TestResult::Panic { path, .. } => {
                panicked += 1;
                ("Panicked", path, "runner panicked")
            }
        };
        let path = path.display().to_string().replace('\\', "/");
        diagnostics.push(format!("AST Round Trip {status}: {path} - {error}"));
    }
    let rate = |count| {
        if total == 0 {
            0.0
        } else {
            count as f64 / total as f64 * 100.0
        }
    };
    let mut snapshot = format!(
        "\nAST Round Trip (preserve_parens={preserve_parens})\nAST Round Trip Passed: {passed}/{total} ({:.2}%)\nAST Round Trip Failed: {failed}/{total} ({:.2}%)\nAST Round Trip Skipped: {skipped}/{total} ({:.2}%)\nAST Round Trip Panicked: {panicked}/{total} ({:.2}%)\n",
        rate(passed),
        rate(failed),
        rate(skipped),
        rate(panicked),
    );
    diagnostics.sort();
    for diagnostic in diagnostics {
        snapshot.push_str(&diagnostic);
        snapshot.push('\n');
    }
    snapshot
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn independent_round_trip_counts() {
        let codegen = [TestResult::Passed {
            path: "changed.js".into(),
            negative: false,
        }];
        let round_trip = [
            TestResult::Failed {
                path: "changed.js".into(),
                negative: false,
                error: "operator changed".into(),
            },
            TestResult::Passed {
                path: "valid.js".into(),
                negative: false,
            },
            TestResult::Skipped {
                path: "invalid.js".into(),
                negative: false,
                reason: "input parse error".into(),
            },
            TestResult::Panic {
                path: "panic.js".into(),
                negative: false,
            },
        ];
        let mut snapshot = to_snapshot("codegen", "sample", &codegen);
        snapshot.push_str(&to_round_trip_snapshot(&round_trip, false));
        assert!(snapshot.contains("Positive Passed: 1/1 (100.00%)"));
        assert!(snapshot.contains("AST Round Trip Passed: 1/4 (25.00%)"));
        assert!(snapshot.contains("AST Round Trip Failed: 1/4 (25.00%)"));
        assert!(snapshot.contains("AST Round Trip Skipped: 1/4 (25.00%)"));
        assert!(snapshot.contains("AST Round Trip Panicked: 1/4 (25.00%)"));
        assert!(snapshot.contains("AST Round Trip Failed: changed.js - operator changed"));
        assert!(snapshot.contains("AST Round Trip Skipped: invalid.js - input parse error"));
        assert!(snapshot.contains("AST Round Trip (preserve_parens=false)"));
        let with_parens = to_round_trip_snapshot(&round_trip, true);
        assert!(with_parens.contains("AST Round Trip (preserve_parens=true)"));
        assert!(!has_panics(&codegen));
        assert!(has_panics(&round_trip));
    }

    #[test]
    fn parser_snapshot() {
        let result = TestResult::Passed {
            path: "valid.js".into(),
            negative: false,
        };
        let snapshot = to_snapshot("parser", "sample", &[result]);
        assert!(snapshot.contains("Positive Passed: 1/1 (100.00%)"));
        assert!(!snapshot.contains("Round Trip"));
    }
}
