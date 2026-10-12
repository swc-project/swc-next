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
