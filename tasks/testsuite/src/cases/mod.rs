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
    fn lang(&self) -> Lang;
    fn expects_error(&self) -> bool;

    fn relative_path(&self) -> &Path {
        self.path().strip_prefix(crate_root()).unwrap()
    }
}

pub fn fixtures() -> PathBuf {
    crate_root().join("fixtures")
}
