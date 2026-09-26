use std::{
    fs,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use swc_next_ecma_ast::{Lang, SourceType};

use crate::{cases::Case, util::crate_root};

pub struct RegressionCase {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    expects_error: bool,
}

impl Case for RegressionCase {
    const NAME: &'static str = "regressions";

    fn read() -> Vec<Self> {
        let root = crate_root().join("fixtures-manual/regressions");
        let mut cases = Vec::new();
        for entry in WalkDir::new(root) {
            let entry = entry.expect("regression fixture directory must be readable");
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let source_type = match path.extension().and_then(|extension| extension.to_str()) {
                Some("js") => SourceType::Script,
                Some("mjs") => SourceType::Module,
                _ => continue,
            };
            let expects_error = path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| stem.ends_with(".negative"));
            let code =
                fs::read_to_string(&path).expect("regression fixture must be readable UTF-8");
            cases.push(Self {
                path,
                code,
                source_type,
                expects_error,
            });
        }
        cases.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        cases
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn name(&self) -> &str {
        self.source_type.as_str()
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn source_type(&self) -> SourceType {
        self.source_type
    }

    fn lang(&self) -> Lang {
        Lang::Js
    }

    fn expects_error(&self) -> bool {
        self.expects_error
    }
}
