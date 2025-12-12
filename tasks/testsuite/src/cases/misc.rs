use std::{
    fs::read_dir,
    path::{Path, PathBuf},
};

use swc_experimental_ecma_parser::{EsSyntax, Syntax};

use crate::cases::{Case, fixtures};

pub struct MiscCase {
    path: PathBuf,
    code: String,
    should_fail: bool,
}

impl MiscCase {
    pub fn read() -> Vec<Self> {
        let mut pass_cases = Vec::new();
        let mut fail_cases = Vec::new();

        #[allow(clippy::single_element_loop)]
        for test_path in &[fixtures().join("misc-parser")] {
            let test_path = test_path.join(test_path);
            pass_cases.extend(read_dir(test_path.join("pass")).unwrap());
            fail_cases.extend(read_dir(test_path.join("fail")).unwrap());
        }

        let mut cases = Vec::new();
        for pass_case in pass_cases {
            let pass_case = pass_case.unwrap();
            let path = pass_case.path();
            let code = std::fs::read_to_string(&path).unwrap();
            cases.push(MiscCase {
                path,
                code,
                should_fail: false,
            });
        }

        for fail_case in fail_cases {
            let fail_case = fail_case.unwrap();
            let path = fail_case.path();
            let code = std::fs::read_to_string(&path).unwrap();
            cases.push(MiscCase {
                path,
                code,
                should_fail: true,
            });
        }

        cases
    }
}

impl Case for MiscCase {
    fn path(&self) -> &Path {
        &self.path
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn should_fail(&self) -> bool {
        self.should_fail
    }

    fn should_ignore(&self) -> bool {
        self.should_fail() && IGNORED_FAIL_TESTS.contains(&self.filename().as_str())
    }

    fn syntax(&self) -> Syntax {
        let ext = self.ext();
        match ext.as_str() {
            "js" | "cjs" | "mjs" | "jsx" => {
                let mut es = EsSyntax::default();
                if ext == "jsx" {
                    es.jsx = true;
                }
                if self.code.contains("// @decorators") {
                    es.decorators = true;
                }
                if self.code.contains("// @decorators_before_export") {
                    es.decorators_before_export = true;
                }
                if self.code.contains("// @export_default_from") {
                    es.export_default_from = true;
                }
                if self.code.contains("// @allow_super_outside_method") {
                    es.allow_super_outside_method = true;
                }
                Syntax::Es(es)
            }
            "ts" | "cts" | "mts" | "tsx" => Syntax::Es(EsSyntax::default()),
            _ => unreachable!(),
        }
    }
}

const IGNORED_FAIL_TESTS: &[&str] = &[
    // SWC passes these cases, which seems like bugs.
    "errors-issue-387-4-input.jsx",
    "errors-html-comment-input.jsx",
];
