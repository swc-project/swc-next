use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

pub enum TestResult {
    Skipped {
        path: PathBuf,
    },
    Passed {
        path: PathBuf,
        negative: bool,
    },
    Failed {
        path: PathBuf,
        error: String,
        negative: bool,
    },
    Panic {
        path: PathBuf,
        negative: bool,
    },
}

impl Display for TestResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TestResult::Skipped { path } => write!(f, "Skipped: {}", path.display()),
            TestResult::Passed { path, .. } => write!(f, "Passed: {}", path.display()),
            TestResult::Failed { path, error, .. } => {
                write!(f, "Failed: {} - {}", path.display(), error)
            }
            TestResult::Panic { path, .. } => write!(f, "Panic: {}", path.display()),
        }
    }
}

pub fn to_snapshot(runner: &str, cases: &str, results: &[TestResult]) -> String {
    let total = results.len();
    let mut positive = 0;
    let mut negative = 0;
    let mut positive_passed = 0;
    let mut negative_passed = 0;
    let mut panicked = 0;
    let mut skipped = 0;
    let mut diagnostics = Vec::new();

    for result in results {
        let is_negative = match result {
            TestResult::Passed { negative, .. }
            | TestResult::Failed { negative, .. }
            | TestResult::Panic { negative, .. } => *negative,
            TestResult::Skipped { .. } => false,
        };
        if is_negative {
            negative += 1;
        } else {
            positive += 1;
        }

        match result {
            TestResult::Skipped { .. } => skipped += 1,
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
    if skipped != 0 {
        snapshot.push_str(&format!(
            "Skipped: {skipped}/{total} ({:.2}%)\n",
            rate(skipped, total),
        ));
    }
    for diagnostic in diagnostics {
        snapshot.push_str(&diagnostic);
        snapshot.push('\n');
    }
    snapshot
}
