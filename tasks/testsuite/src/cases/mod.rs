use std::path::{Path, PathBuf};

use crate::util::crate_root;

pub mod test262;

pub trait Case: Sync {
    fn path(&self) -> &Path;
    fn code(&self) -> &str;

    fn relative_path(&self) -> &Path {
        self.path().strip_prefix(crate_root()).unwrap()
    }
}

pub fn fixtures() -> PathBuf {
    crate_root().join("fixtures")
}
