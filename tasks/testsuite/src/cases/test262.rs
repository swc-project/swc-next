use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use saphyr::{LoadableYamlNode, Yaml};

use crate::cases::{Case, fixtures};

/// https://github.com/evanw/esbuild/blob/main/scripts/test262.js
pub struct Test262Case {
    path: PathBuf,
    code: String,
    meta: Meta,
}

#[derive(Default)]
struct Meta {
    negative: Option<MetaNegative>,
}

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
            let Some(meta) = parse_meta(&code) else {
                continue;
            };
            cases.push(Self {
                code,
                path: file.path(),
                meta,
            });
        }

        cases
    }
}

fn parse_meta(source: &str) -> Option<Meta> {
    let meta_start = source.find("/*---")?;
    let meta_end = source.find("--*/")?;
    let meta_str = &source[meta_start + 5..meta_end];

    let yaml = Yaml::load_from_str(meta_str).unwrap_or_default();
    let Some(yaml) = yaml.first() else {
        return Some(Meta::default());
    };

    Some(Meta {
        negative: yaml
            .as_mapping_get("negative")
            .filter(|yaml| !yaml.is_null() && !yaml.is_badvalue())
            .map(|yaml| MetaNegative {
                phase: Phase::from(yaml["phase"].as_str().unwrap()),
            }),
    })
}

impl Case for Test262Case {
    fn path(&self) -> &Path {
        &self.path
    }

    fn code(&self) -> &str {
        &self.code
    }

    fn should_ignore(&self) -> bool {
        IGNORED_TESTS
            .iter()
            .any(|ignore| self.path.to_string_lossy().contains(ignore))
    }

    fn should_fail(&self) -> bool {
        self.meta
            .negative
            .as_ref()
            .map(|neg| neg.phase == Phase::Parse)
            .unwrap_or(false)
    }
}

const IGNORED_TESTS: &[&str] = &[
    // Should be fixed
    "fixtures/test262/test/built-ins/Temporal/PlainMonthDay/argument-invalid.js",
    "fixtures/test262/test/built-ins/Temporal/PlainYearMonth/argument-invalid.js",
    "fixtures/test262/test/built-ins/Array/prototype/reverse/length-exceeding-integer-limit-with-proxy.js",
    "fixtures/test262/test/built-ins/Iterator/zipKeyed/iterables-iteration.js",
    "fixtures/test262/test/language/comments/hashbang/line-terminator-paragraph-separator.js",
    "fixtures/test262/test/language/comments/hashbang/line-terminator-line-separator.js",
    "fixtures/test262/test/language/literals/bigint/numeric-separators/numeric-separator-literal-hil-hd-nsl-hd-err.js",
    "fixtures/test262/test/language/literals/bigint/numeric-separators/numeric-separator-literal-oil-od-nsl-od-err.js",
    "fixtures/test262/test/language/literals/bigint/numeric-separators/numeric-separator-literal-bil-bd-nsl-bd-err.js",
    "fixtures/test262/test/language/literals/string/S7.8.4_A4.2_T6.js",
    "fixtures/test262/test/language/literals/string/S7.8.4_A4.2_T7.js",
    "fixtures/test262/test/language/literals/string/S7.8.4_A4.2_T8.js",
    "fixtures/test262/test/language/literals/string/S7.8.4_A4.2_T5.js",
];
