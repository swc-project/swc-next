use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
    time::Duration,
};

use jwalk::WalkDir;
use saphyr::{LoadableYamlNode, Yaml};
use swc_experimental_ecma_ast::{Lang, SourceType};

use crate::cases::{Case, fixtures};

/// https://github.com/evanw/esbuild/blob/main/scripts/test262.js
pub struct Test262Case {
    path: PathBuf,
    code: String,
    source_type: SourceType,
    expects_error: bool,
    name: &'static str,
}

#[derive(Debug, Default)]
struct Meta {
    negative: Option<MetaNegative>,
    module: bool,
    only_strict: bool,
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
        let mut cases = Vec::new();
        for file in WalkDir::new(fixtures().join("test262").join("test")).parallelism(
            jwalk::Parallelism::RayonDefaultPool {
                busy_timeout: Duration::from_secs(1),
            },
        ) {
            let file = file.unwrap();
            if !file.file_type().is_file() {
                continue;
            }

            let path = file.path().to_string_lossy().replace('\\', "/");
            if !path.ends_with(".js")
                || path.contains("_FIXTURE")
                || path.contains("test262/test/staging")
            {
                continue;
            }

            let code = read_to_string(path).unwrap();
            let Some(meta) = parse_meta(&code) else {
                continue;
            };
            let expects_error = meta
                .negative
                .as_ref()
                .is_some_and(|negative| matches!(negative.phase, Phase::Parse | Phase::Early));
            let path = file.path();
            if meta.module {
                cases.push(Self {
                    path,
                    code,
                    source_type: SourceType::Module,
                    expects_error,
                    name: "module",
                });
                continue;
            }

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
        }

        cases
    }
}

fn parse_meta(source: &str) -> Option<Meta> {
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
