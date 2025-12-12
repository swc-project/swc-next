use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

pub enum TestResult {
    Passed { path: PathBuf },
    Failed { path: PathBuf, error: String },
    Ignored { path: PathBuf },
    Panic { path: PathBuf },
}

impl Display for TestResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            TestResult::Passed { path } => write!(f, "Passed: {}", path.display()),
            TestResult::Failed { path, error } => {
                write!(f, "Failed: {} - {}", path.display(), error)
            }
            TestResult::Ignored { path } => write!(f, "Ignored: {}", path.display()),
            TestResult::Panic { path } => write!(f, "Panic: {}", path.display()),
        }
    }
}
