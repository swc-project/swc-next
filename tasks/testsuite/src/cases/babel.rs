use std::{
    fs::{self, read_to_string},
    io,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use rayon::prelude::*;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_codegen::Options as CodegenOptions;
use swc_next_ecma_transformer::TransformerOptions;

use crate::cases::{Case, TransformCase, fixtures};

pub mod options;

use options::BabelOptions;

pub(super) const INPUT_FILE_NAMES: &[&str] = &[
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
pub struct BabelTransformCase {
    pub path: PathBuf,
    pub name: String,
    pub input: String,
    pub output: String,
    pub options: BabelOptions,
}

struct BabelTransformCasePaths {
    input: PathBuf,
    output: PathBuf,
}

impl BabelTransformCase {
    pub fn read() -> Vec<Self> {
        Self::try_read().unwrap()
    }

    fn try_read() -> io::Result<Vec<Self>> {
        let packages_dir = fixtures().join("babel/packages");
        let mut case_paths = Vec::new();

        for package in fs::read_dir(&packages_dir)? {
            let package = package?;
            if !package.file_type()?.is_dir() {
                continue;
            }

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
                let case_dir = input_path.parent().unwrap();
                let Some(output_path) = OUTPUT_FILE_NAMES
                    .iter()
                    .map(|name| case_dir.join(name))
                    .find(|path| path.is_file())
                else {
                    continue;
                };

                case_paths.push(BabelTransformCasePaths {
                    input: input_path,
                    output: output_path,
                });
            }
        }

        let mut cases = case_paths
            .into_par_iter()
            .map(|paths| {
                let case_dir = paths.input.parent().unwrap();
                let options = BabelOptions::from_test_path(case_dir);
                let name = case_dir
                    .strip_prefix(&packages_dir)
                    .unwrap_or(case_dir)
                    .to_string_lossy()
                    .replace('\\', "/");
                let input = read_to_string(&paths.input)?;
                let output = read_to_string(&paths.output)?;
                Ok(Self {
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

    pub fn source_type(&self) -> SourceType {
        source_type(&self.path, &self.options)
    }

    pub fn lang(&self) -> Lang {
        lang(&self.path, &self.options)
    }
}

impl Case for BabelTransformCase {
    const NAME: &'static str = "babel";

    fn read() -> Vec<Self> {
        BabelTransformCase::read()
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
        BabelTransformCase::source_type(self)
    }

    fn lang(&self) -> Lang {
        BabelTransformCase::lang(self)
    }

    fn expects_error(&self) -> bool {
        false
    }
}

impl TransformCase for BabelTransformCase {
    fn supports_options(&self) -> bool {
        self.options.plugins.errors.is_empty()
            && self.options.plugins.unsupported.is_empty()
            && self.options.presets.errors.is_empty()
            && self.options.presets.unsupported.is_empty()
    }

    fn expected_output(&self) -> Option<&str> {
        Some(&self.output)
    }

    fn transformer_options(&self) -> TransformerOptions {
        self.options.clone().into()
    }

    fn codegen_options(&self) -> CodegenOptions {
        self.options.codegen_options()
    }
}

pub(super) fn source_type(path: &Path, options: &BabelOptions) -> SourceType {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("mjs" | "mts") => return SourceType::Module,
        Some("cjs" | "cts") => return SourceType::CommonJs,
        _ => {}
    }

    match options.source_type.as_deref() {
        Some("script") => SourceType::Script,
        Some("commonjs") => SourceType::CommonJs,
        Some("module") => SourceType::Module,
        Some("unambiguous") => SourceType::Unambiguous,
        // Babel's transform fixture harness treats plain files as scripts while
        // still accepting module syntax. `Unambiguous` is the closest match in
        // swc-next's mutually exclusive source-type model.
        None => SourceType::Unambiguous,
        Some(_) => SourceType::Module,
    }
}

pub(super) fn lang(path: &Path, options: &BabelOptions) -> Lang {
    let path_lang = Lang::from_path(&path.to_string_lossy());
    if path_lang == Lang::Dts || options.is_typescript_definition() {
        return Lang::Dts;
    }

    let typescript = matches!(path_lang, Lang::Ts | Lang::Tsx)
        || options.is_typescript()
        || options.plugins.typescript.is_some();
    let jsx = matches!(path_lang, Lang::Jsx | Lang::Tsx) || options.is_jsx();
    match (typescript, jsx) {
        (true, true) => Lang::Tsx,
        (true, false) => Lang::Ts,
        (false, true) => Lang::Jsx,
        (false, false) => Lang::Js,
    }
}
