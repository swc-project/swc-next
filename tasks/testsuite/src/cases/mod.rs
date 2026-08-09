use std::path::{Path, PathBuf};

use swc_next_ecma_ast::{Lang, SourceType};

use crate::util::crate_root;

pub mod babel;
pub mod babel_parser;
pub mod test262;
pub mod typescript;

pub trait Case: Sized + Sync {
    const NAME: &'static str;

    fn read() -> Vec<Self>;
    fn path(&self) -> &Path;
    fn name(&self) -> &str;
    fn code(&self) -> &str;
    fn source_type(&self) -> SourceType;
    fn expected_source_type(&self) -> Option<SourceType> {
        None
    }
    fn lang(&self) -> Lang;
    fn expects_error(&self) -> bool;
    /// Whether a case expected to succeed also asserts that semantic analysis
    /// produces no early-error diagnostics.
    fn asserts_semantic_success(&self) -> bool {
        true
    }

    fn relative_path(&self) -> &Path {
        self.path().strip_prefix(crate_root()).unwrap()
    }
}

pub fn fixtures() -> PathBuf {
    crate_root().join("fixtures")
}
