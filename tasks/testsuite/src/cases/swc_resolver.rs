use std::{
    collections::BTreeMap,
    fs::read_to_string,
    path::{Path, PathBuf},
};

use rayon::prelude::*;
use serde::Deserialize;
use swc_next_ecma_ast::{Lang, SourceType};

use crate::{
    cases::{Case, fixtures, read_json, swc_conformance},
    util::workspace_root,
};

pub struct SwcResolverCase {
    path: PathBuf,
    relative_path: PathBuf,
    name: String,
    code: String,
    expected: BTreeMap<u32, usize>,
    lang: Lang,
    source_type: SourceType,
}

#[derive(Deserialize)]
struct Input {
    path: PathBuf,
    snapshot: PathBuf,
    prefix: String,
    module: bool,
    jsx: bool,
}

impl SwcResolverCase {
    pub fn expected(&self) -> &BTreeMap<u32, usize> {
        &self.expected
    }

    fn read_cases() -> Vec<Self> {
        let workspace_root = workspace_root().expect("workspace root should be discoverable");
        let root = swc_conformance();
        let oracle_root = root.join("tests/resolver");
        assert!(
            oracle_root.is_dir(),
            "missing resolver snapshots: {}",
            oracle_root.display()
        );
        let inputs: Vec<Input> = read_json(&root.join("fixtures-json/runners/swc.json"));
        inputs
            .into_par_iter()
            .filter_map(|input| {
                let oracle_path = oracle_root.join(&input.snapshot);
                // SWC omits resolver snapshots for inputs its parser rejects.
                if !oracle_path.is_file() {
                    return None;
                }
                Some(Self::read_case(&workspace_root, input, &oracle_path))
            })
            .collect()
    }

    fn read_case(workspace_root: &Path, input: Input, oracle_path: &Path) -> Self {
        let path = fixtures().join(&input.path);
        let relative_path = path.strip_prefix(workspace_root).unwrap().to_path_buf();
        let source = read_to_string(&path).unwrap();
        let code = format!("{}{source}", input.prefix);
        let expected = read_json(oracle_path);
        let lang = if input.jsx { Lang::Jsx } else { Lang::Js };
        let source_type = if input.module {
            SourceType::Module
        } else {
            SourceType::Unambiguous
        };
        Self {
            path,
            relative_path,
            name: input.snapshot.to_string_lossy().replace('\\', "/"),
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
