use std::{
    fs::{self, read_to_string},
    io,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use swc_experimental_ecma_ast::{Lang, SourceType};

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

impl BabelCase {
    pub fn read() -> Vec<Self> {
        Self::try_read().unwrap_or_else(|error| panic!("failed to read Babel fixtures: {error}"))
    }

    pub fn try_read() -> io::Result<Vec<Self>> {
        let packages_dir = fixtures().join("babel/packages");
        let mut cases = Vec::new();

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

                let options = BabelOptions::from_test_path(case_dir)?;

                let name = case_dir
                    .strip_prefix(&packages_dir)
                    .unwrap_or(case_dir)
                    .to_string_lossy()
                    .replace('\\', "/");
                let input = read_to_string(&input_path)?;
                let output = read_to_string(output_path)?;
                cases.push(Self {
                    package_name: package_name.clone(),
                    path: input_path,
                    name,
                    input,
                    output,
                    options,
                });
            }
        }

        cases.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        Ok(cases)
    }

    pub fn source_type(&self) -> SourceType {
        match self
            .path
            .extension()
            .and_then(|extension| extension.to_str())
        {
            Some("mjs" | "mts") => return SourceType::Module,
            Some("cjs" | "cts") => return SourceType::Script,
            _ => {}
        }

        match self.options.source_type.as_deref() {
            Some("script" | "commonjs") => SourceType::Script,
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
