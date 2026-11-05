use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
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
        let workspace_root = workspace_root().unwrap();
        let oracle_root = fixtures().join("swc-conformance/tests/resolver");
        let oracle_paths = read_oracle_paths(&oracle_root);

        oracle_paths
            .into_par_iter()
            .map(|oracle_path| Self::read_case(&workspace_root, &oracle_root, oracle_path))
            .collect()
    }

    fn read_case(workspace_root: &Path, oracle_root: &Path, oracle_path: PathBuf) -> Self {
        let oracle_relative = oracle_path.strip_prefix(oracle_root).unwrap();
        let mut components = oracle_relative.components();
        let suite = components.next().unwrap().as_os_str();
        let fixture_root = workspace_root.join("tasks/testsuite/fixtures").join(suite);
        let relative_source = components.as_path().with_extension("");
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

fn read_oracle_paths(root: &Path) -> Vec<PathBuf> {
    assert!(
        root.is_dir(),
        "missing resolver fixtures: {}",
        root.display()
    );
    let entries = WalkDir::new(root).into_iter().map(Result::unwrap);
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry.path();
        let snapshot = path.extension().is_some_and(|ext| ext == "snap");
        if entry.file_type().is_file() && snapshot {
            paths.push(path);
        }
    }
    paths.sort_unstable();
    paths
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
