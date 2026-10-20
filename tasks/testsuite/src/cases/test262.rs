use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use rayon::prelude::*;
use saphyr::{LoadableYamlNode, Yaml};
use swc_next_ecma_ast::{Lang, SourceType};

use crate::{
    cases::{Case, fixtures},
    util::crate_root,
};

/// https://github.com/evanw/esbuild/blob/main/scripts/test262.js
pub struct Test262Case {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    expects_error: bool,
    name: &'static str,
}

#[derive(Debug, Default)]
pub(super) struct Meta {
    negative: Option<MetaNegative>,
    pub(super) module: bool,
    pub(super) only_strict: bool,
    no_strict: bool,
    raw: bool,
}

#[derive(Debug)]
struct MetaNegative {
    phase: Phase,
}

#[derive(Debug, PartialEq, Eq)]
enum Phase {
    Parse,
    Early,
    Resolution,
    Runtime,
}

impl From<&str> for Phase {
    fn from(value: &str) -> Self {
        match value {
            "parse" => Self::Parse,
            "early" => Self::Early,
            "resolution" => Self::Resolution,
            "runtime" => Self::Runtime,
            _ => panic!("invalid meta phase"),
        }
    }
}

impl Test262Case {
    fn read_cases() -> Vec<Self> {
        let roots = [
            fixtures().join("test262/test"),
            crate_root().join("fixtures-test262-legacy"),
        ];
        roots
            .into_iter()
            .flat_map(WalkDir::new)
            .filter_map(|file| {
                let file = file.unwrap();
                if !file.file_type().is_file() {
                    return None;
                }

                let path = file.path();
                let normalized_path = path.to_string_lossy().replace('\\', "/");
                // These Annex B cases require call expressions to be accepted as assignment
                // targets in non-strict mode, which this parser intentionally does not support.
                (normalized_path.ends_with(".js")
                    && !normalized_path.contains("_FIXTURE")
                    && !normalized_path.contains("test262/test/staging")
                    && !normalized_path
                        .contains("test262/test/annexB/language/expressions/assignmenttargettype"))
                .then_some(path)
            })
            .collect::<Vec<_>>()
            .into_par_iter()
            .flat_map(Self::read_file)
            .collect()
    }

    pub(super) fn read_file(path: PathBuf) -> Vec<Self> {
        let code = read_to_string(&path).unwrap();
        let Some(meta) = parse_meta(&code) else {
            return Vec::new();
        };
        let expects_error = meta
            .negative
            .as_ref()
            .is_some_and(|negative| matches!(negative.phase, Phase::Parse | Phase::Early));
        if meta.module {
            return vec![Self {
                path,
                code,
                source_type: SourceType::Module,
                expects_error,
                name: "module",
            }];
        }

        let mut cases = Vec::with_capacity(2);
        if !meta.only_strict {
            cases.push(Self {
                path: path.clone(),
                code: code.clone(),
                source_type: SourceType::Script,
                expects_error,
                name: "sloppy",
            });
        }
        if !meta.no_strict && !meta.raw {
            cases.push(Self {
                path,
                code: format!("\"use strict\";\n{code}"),
                source_type: SourceType::Script,
                expects_error,
                name: "strict",
            });
        }
        cases
    }
}

pub(super) fn parse_meta(source: &str) -> Option<Meta> {
    let meta_start = source.find("/*---")?;
    let meta_end = source.find("---*/")?;
    let meta_str = &source[meta_start + 5..meta_end];

    let yaml = Yaml::load_from_str(meta_str).unwrap_or_default();
    let Some(yaml) = yaml.first() else {
        return Some(Meta::default());
    };

    let flags = yaml
        .as_mapping_get("flags")
        .and_then(Yaml::as_sequence)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let has_flag = |expected| flags.iter().any(|flag| flag.as_str() == Some(expected));

    Some(Meta {
        negative: yaml
            .as_mapping_get("negative")
            .filter(|yaml| !yaml.is_null() && !yaml.is_badvalue())
            .map(|yaml| {
                if yaml.as_str() == Some("SyntaxError") {
                    return MetaNegative {
                        phase: Phase::Parse,
                    };
                }

                MetaNegative {
                    phase: yaml
                        .as_mapping_get("phase")
                        .map(|p| Phase::from(p.as_str().unwrap()))
                        .unwrap_or(Phase::Parse),
                }
            }),
        module: has_flag("module"),
        only_strict: has_flag("onlyStrict"),
        no_strict: has_flag("noStrict"),
        raw: has_flag("raw"),
    })
}

impl Case for Test262Case {
    const NAME: &'static str = "test262";

    fn read() -> Vec<Self> {
        Self::read_cases()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn name(&self) -> &str {
        self.name
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
