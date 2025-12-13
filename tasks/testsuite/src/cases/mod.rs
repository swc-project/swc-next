use std::path::{Path, PathBuf};

use swc_experimental_ecma_parser::Syntax;

use crate::util::crate_root;

pub mod misc;
pub mod test262;
pub mod test262_parser;

pub enum IsModule {
    Module,
    Script,
    Unknown,
    Skip,
}

pub trait Case: Sync {
    fn path(&self) -> &Path;
    fn code(&self) -> &str;
    fn should_fail(&self) -> bool;

    fn should_ignore(&self) -> bool {
        false
    }

    fn is_module(&self) -> IsModule {
        match self.ext().as_str() {
            "cjs" => IsModule::Script,
            "mjs" => IsModule::Module,
            "js" | "jsx" => IsModule::Unknown,
            "ts" | "tsx" => IsModule::Skip,
            _ => unreachable!(),
        }
    }

    fn syntax(&self) -> Syntax {
        Syntax::default()
    }

    fn ext(&self) -> String {
        self.path()
            .extension()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }

    fn filename(&self) -> String {
        self.path()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    fn relative_path(&self) -> &Path {
        self.path().strip_prefix(crate_root()).unwrap()
    }
}

pub fn fixtures() -> PathBuf {
    crate_root().join("fixtures")
}
