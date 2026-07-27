use std::{
    fs::{self, read_to_string},
    io,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use rayon::prelude::*;
use swc_next_ecma_ast::{Lang, SourceType};

use crate::cases::{Case, fixtures};

pub mod options;

use options::BabelOptions;

const INPUT_FILE_NAMES: &[&str] = &[
    "input.js",
    "input.jsx",
    "input.mjs",
    "input.cjs",
    "input.ts",
    "input.tsx",
    "input.mts",
    "input.cts",
    "input.d.ts",
];

const OUTPUT_FILE_NAMES: &[&str] = &[
    "output.js",
    "output.jsx",
    "output.mjs",
    "output.cjs",
    "output.ts",
    "output.tsx",
    "output.mts",
    "output.cts",
];

#[derive(Debug)]
pub struct BabelCase {
    pub package_name: String,
    pub path: PathBuf,
    pub name: String,
    pub input: String,
    pub output: String,
    pub options: BabelOptions,
}

struct BabelCasePaths {
    package_name: String,
    input: PathBuf,
    output: PathBuf,
}

impl BabelCase {
    pub fn read() -> Vec<Self> {
        Self::try_read().unwrap_or_else(|error| panic!("failed to read Babel fixtures: {error}"))
    }

    pub fn try_read() -> io::Result<Vec<Self>> {
        let packages_dir = fixtures().join("babel/packages");
        let mut case_paths = Vec::new();

        for package in fs::read_dir(&packages_dir)? {
            let package = package?;
            if !package.file_type()?.is_dir() {
                continue;
            }

            let package_name = package.file_name().to_string_lossy().into_owned();
            let fixture_dir = package.path().join("test/fixtures");
            if !fixture_dir.is_dir() {
                continue;
            }

            for entry in WalkDir::new(&fixture_dir) {
                let Ok(entry) = entry else { continue };
                if !entry.file_type().is_file()
                    || !INPUT_FILE_NAMES.contains(&entry.file_name().to_string_lossy().as_ref())
                {
                    continue;
                }

                let input_path = entry.path();
                let Some(case_dir) = input_path.parent() else {
                    continue;
                };
                let Some(output_path) = OUTPUT_FILE_NAMES
                    .iter()
                    .map(|name| case_dir.join(name))
                    .find(|path| path.is_file())
                else {
                    continue;
                };

                case_paths.push(BabelCasePaths {
                    package_name: package_name.clone(),
                    input: input_path,
                    output: output_path,
                });
            }
        }

        let mut cases = case_paths
            .into_par_iter()
            .map(|paths| {
                let case_dir = paths.input.parent().unwrap();
                let options = BabelOptions::from_test_path(case_dir)?;
                let name = case_dir
                    .strip_prefix(&packages_dir)
                    .unwrap_or(case_dir)
                    .to_string_lossy()
                    .replace('\\', "/");
                let input = read_to_string(&paths.input)?;
                let output = read_to_string(paths.output)?;
                Ok(Self {
                    package_name: paths.package_name,
                    path: paths.input,
                    name,
                    input,
                    output,
                    options,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        cases.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        Ok(cases)
    }

    pub fn is_proposal(&self) -> bool {
        self.package_name.starts_with("babel-plugin-proposal-")
    }

    pub fn is_flow(&self) -> bool {
        self.name.to_ascii_lowercase().contains("flow")
            || self
                .options
                .plugins
                .unsupported
                .iter()
                .chain(&self.options.presets.unsupported)
                .any(|name| name.to_ascii_lowercase().contains("flow"))
    }

    pub fn has_non_standard_syntax(&self) -> bool {
        const NON_STANDARD_SYNTAX_PATHS: &[&str] = &[
            "/async-do-expressions/",
            "/do-expression",
            "/DoExpression",
            "module-expression",
            "/ModuleExpression",
            "/discard-binding",
            "/throw-expression",
            "/ArgumentPlaceholder",
            "/BindExpression",
            "/TopicReference-",
            "/misc/placeholders",
            "/misc/V8IntrinsicIdentifier",
        ];

        NON_STANDARD_SYNTAX_PATHS
            .iter()
            .any(|path| self.name.contains(path))
    }

    pub fn source_type(&self) -> SourceType {
        match self
            .path
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("mjs" | "mts") => return SourceType::Module,
            Some("cjs" | "cts") => return SourceType::CommonJs,
            _ => {}
        }

        match self.options.source_type.as_deref() {
            Some("script") => SourceType::Script,
            Some("commonjs") => SourceType::CommonJs,
            Some("module" | "unambiguous") | None => SourceType::Module,
            Some(_) => SourceType::Module,
        }
    }

    pub fn lang(&self) -> Lang {
        let path_lang = Lang::from_path(&self.path.to_string_lossy());
        if path_lang == Lang::Dts || self.options.is_typescript_definition() {
            return Lang::Dts;
        }

        let typescript = matches!(path_lang, Lang::Ts | Lang::Tsx)
            || self.options.is_typescript()
            || self.options.plugins.typescript;
        let jsx = matches!(path_lang, Lang::Jsx | Lang::Tsx) || self.options.is_jsx();
        match (typescript, jsx) {
            (true, true) => Lang::Tsx,
            (true, false) => Lang::Ts,
            (false, true) => Lang::Jsx,
            (false, false) => Lang::Js,
        }
    }
}

impl Case for BabelCase {
    const NAME: &'static str = "babel";

    fn read() -> Vec<Self> {
        BabelCase::read()
            .into_iter()
            .filter(|case| {
                !case.is_proposal() && !case.is_flow() && !case.has_non_standard_syntax()
            })
            .collect()
    }

    fn path(&self) -> &Path {
        &self.path
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn code(&self) -> &str {
        &self.input
    }

    fn source_type(&self) -> SourceType {
        BabelCase::source_type(self)
    }

    fn lang(&self) -> Lang {
        BabelCase::lang(self)
    }

    fn expects_error(&self) -> bool {
        // Babel's `throws` describes transform-time failures, while the generic
        // conformance runners only use this flag for parse and early errors.
        false
    }
}
