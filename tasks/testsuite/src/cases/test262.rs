use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::cases::{Case, fixtures};

/// https://github.com/evanw/esbuild/blob/main/scripts/test262.js
pub struct Test262Case {
    path: PathBuf,
    code: String,
    should_fail: bool,
}

impl Test262Case {
    pub fn read() -> Vec<Self> {
        let mut cases = Vec::new();
        for file in WalkDir::new(fixtures().join("test262")) {
            let file = file.unwrap();
            if !file.file_type().is_file() {
                continue;
            }

            let path = file.path().to_string_lossy().to_string();
            if !path.ends_with(".js")
                || path.contains("_FIXTURE")
                || path.contains("test262/test/staging")
            {
                continue;
            }

            let code = read_to_string(path).unwrap();
            cases.push(Self {
                code,
                path: file.into_path(),
                should_fail: false,
            });
        }

        cases
    }
}

impl Case for Test262Case {
    fn path(&self) -> &Path {
        &self.path
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn should_fail(&self) -> bool {
        self.should_fail
    }
}
