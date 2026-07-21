use std::path::{Path, PathBuf};

use swc_experimental_ecma_ast::SourceType;

use crate::util::crate_root;

pub mod test262;

pub struct CaseVariant {
    pub code: String,
    pub source_type: SourceType,
    pub expects_error: bool,
    pub name: &'static str,
}

pub trait Case: Sized + Sync {
    const NAME: &'static str;

    fn read() -> Vec<Self>;
    fn path(&self) -> &Path;
    fn variants(&self) -> Vec<CaseVariant>;

    fn relative_path(&self) -> &Path {
        self.path().strip_prefix(crate_root()).unwrap()
    }
}

pub fn fixtures() -> PathBuf {
    crate_root().join("fixtures")
}
