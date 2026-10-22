use std::{
    fs::read_to_string,
    path::{Component, Path, PathBuf},
};

use jwalk::WalkDir;
use rayon::prelude::*;
use swc_next_ecma_ast::{Lang, SourceType};

use crate::{
    cases::{Case, fixtures, test262::parse_meta},
    util::workspace_root,
};

pub struct SwcResolverCase {
    path: PathBuf,
    relative_path: PathBuf,
    name: String,
    code: String,
    expected: String,
    lang: Lang,
    source_type: SourceType,
}

impl SwcResolverCase {
    pub fn expected(&self) -> &str {
        &self.expected
    }

    fn read_cases() -> Vec<Self> {
        let workspace_root = workspace_root().expect("workspace root should be discoverable");
        let oracle_root = fixtures()
            .join("swc-conformance")
            .join("tests")
            .join("resolver");
        assert!(
            oracle_root.is_dir(),
            "swc-conformance fixtures are missing; run scripts/clone_fixtures.sh first"
        );

        let mut oracle_paths = WalkDir::new(&oracle_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.path())
            .filter(|path| path.to_string_lossy().ends_with(".snap"))
            .collect::<Vec<_>>();
        oracle_paths.sort_unstable();

        oracle_paths
            .into_par_iter()
            .map(|oracle_path| Self::read_case(&workspace_root, &oracle_root, oracle_path))
            .collect()
    }

    fn read_case(workspace_root: &Path, oracle_root: &Path, oracle_path: PathBuf) -> Self {
        let oracle_relative = oracle_path.strip_prefix(oracle_root).unwrap();
        let mut components = oracle_relative.components();
        let Component::Normal(suite) = components.next().unwrap() else {
            panic!("resolver oracle suite should be a normal path component");
        };
        let fixture_root = match suite.to_str() {
            Some("test262") => workspace_root.join("tasks/testsuite/fixtures/test262"),
            Some("swc") => workspace_root.join("tasks/testsuite/fixtures/swc"),
            _ => panic!("unknown resolver oracle suite: {}", suite.display()),
        };
        let relative_snapshot = components.as_path();
        let relative_source = relative_snapshot.to_string_lossy();
        let relative_source = relative_source.strip_suffix(".snap").unwrap();
        let path = fixture_root.join(relative_source);
        let relative_path = path.strip_prefix(workspace_root).unwrap().to_path_buf();
        let name = oracle_relative.display().to_string().replace('\\', "/");
        let mut code = read_to_string(&path).unwrap();
        let expected = read_to_string(&oracle_path).unwrap();
        let lang = if suite == "swc" {
            // swc-conformance enables JSX for every minifier fixture.
            Lang::Jsx
        } else {
            Lang::from_path(&path.to_string_lossy())
        };
        let meta = if suite == "test262" {
            parse_meta(&code)
        } else {
            None
        };
        let source_type = if meta.as_ref().is_some_and(|meta| meta.module) {
            SourceType::Module
        } else {
            SourceType::Unambiguous
        };
        if meta.is_some_and(|meta| meta.only_strict && !meta.module) {
            code = format!("\"use strict\";\n{code}");
        }

        Self {
            path,
            relative_path,
            name,
            code,
            expected,
            lang,
            source_type,
        }
    }
}

impl Case for SwcResolverCase {
    const NAME: &'static str = "swc_resolver";

    fn read() -> Vec<Self> {
        Self::read_cases()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn relative_path(&self) -> &Path {
        &self.relative_path
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn source_type(&self) -> SourceType {
        self.source_type
    }

    fn lang(&self) -> Lang {
        self.lang
    }

    fn expects_error(&self) -> bool {
        false
    }
}
