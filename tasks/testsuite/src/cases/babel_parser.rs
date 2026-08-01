use std::{
    fs::read_to_string,
    io,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use rayon::prelude::*;
use serde::Deserialize;
use swc_next_ecma_ast::{Lang, SourceType};

use crate::cases::{
    Case,
    babel::{INPUT_FILE_NAMES, lang, options::BabelOptions, source_type},
    fixtures,
};

const UNSUPPORTED_PARSER_PATHS: &[&str] = &[
    "/experimental/",
    "/record-and-tuple/",
    "/es-record/",
    "/es-tuple/",
    "/with-pipeline/",
    "/v8intrinsic/",
    "/async-do-expression/",
    "/export-ns-from/",
    "/annex-b/disabled/",
    "/annex-b/enabled/valid-assignment-target-type/",
    "/module-block/",
    "/typescript/arrow-function/arrow-like-in-conditional-2/",
    "/typescript/cast/satisfies-const-error/",
    "/es2022/top-level-await-unambiguous/",
    "/explicit-resource-management/valid-for-await-using-binding-escaped-of-of/",
    "/explicit-resource-management/valid-for-using-binding-escaped-of-of/",
    "/core/categorized/invalid-startindex-and-startline-specified-without-startcolumn/",
    "/core/categorized/startline-and-startcolumn-specified/",
    "/core/categorized/startline-specified/",
    "/core/sourcetype-commonjs/invalid-allowAwaitOutsideFunction-false/",
    "/core/sourcetype-commonjs/invalid-allowNewTargetOutsideFunction-false/",
    "/core/sourcetype-commonjs/invalid-allowReturnOutsideFunction-false/",
    "/core/sourcetype-commonjs/invalid-allowReturnOutsideFunction-true/",
];

pub struct BabelParserCase {
    path: PathBuf,
    name: String,
    input: String,
    options: BabelOptions,
    expects_error: bool,
}

#[derive(Deserialize)]
struct BabelParserOutput {
    errors: Option<Vec<String>>,
}

impl BabelParserCase {
    fn read() -> Vec<Self> {
        Self::try_read().unwrap()
    }

    fn try_read() -> io::Result<Vec<Self>> {
        let fixtures_dir = fixtures().join("babel/packages/babel-parser/test/fixtures");
        let paths = WalkDir::new(&fixtures_dir)
            .into_iter()
            .filter_map(|entry| {
                let entry = entry.ok()?;
                let path = entry.path();
                (entry.file_type().is_file()
                    && INPUT_FILE_NAMES.contains(&entry.file_name().to_string_lossy().as_ref())
                    && is_supported_parser_path(&path))
                .then(|| entry.path())
            })
            .collect::<Vec<_>>();

        let mut cases = paths
            .into_par_iter()
            .map(|path| {
                let case_dir = path.parent().unwrap();
                let options = BabelOptions::from_test_path(case_dir);
                let name = case_dir
                    .strip_prefix(&fixtures_dir)
                    .unwrap_or(case_dir)
                    .to_string_lossy()
                    .replace('\\', "/");
                let expects_error = parser_expects_error(case_dir)?;
                let input = read_to_string(&path)?;
                Ok(Self {
                    path,
                    name,
                    input,
                    options,
                    expects_error,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        cases.retain(Self::is_supported);
        cases.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        Ok(cases)
    }

    fn is_supported(&self) -> bool {
        const UNSUPPORTED_PLUGINS: &[&str] = &[
            "async-do-expression",
            "flow",
            "placeholders",
            "decorators-legacy",
            "recordAndTuple",
        ];

        is_supported_parser_path(&self.path)
            && !self
                .options
                .plugins
                .unsupported
                .iter()
                .any(|plugin| UNSUPPORTED_PLUGINS.contains(&plugin.as_str()))
            && !self.options.allow_await_outside_function
            && !self.options.allow_return_outside_function
            && !self.options.allow_undeclared_exports
            && !self.options.allow_new_target_outside_function
            && !self.options.allow_super_outside_method
            && !self
                .options
                .plugins
                .syntax_typescript
                .is_some_and(|options| options.disallow_ambiguous_jsx_like)
    }
}

fn is_supported_parser_path(path: &Path) -> bool {
    let normalized_path = path.to_string_lossy().replace('\\', "/");
    !UNSUPPORTED_PARSER_PATHS
        .iter()
        .any(|path| normalized_path.contains(path))
}

fn parser_expects_error(case_dir: &Path) -> io::Result<bool> {
    for file_name in ["output.json", "output.extended.json"] {
        let output_path = case_dir.join(file_name);
        if output_path.is_file() {
            let output = serde_json::from_str::<BabelParserOutput>(&read_to_string(&output_path)?)?;
            return Ok(output.errors.is_some_and(|errors| !errors.is_empty()));
        }
    }
    Ok(true)
}

impl Case for BabelParserCase {
    const NAME: &'static str = "babel";

    fn read() -> Vec<Self> {
        BabelParserCase::read()
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
        source_type(&self.path, &self.options)
    }

    fn lang(&self) -> Lang {
        lang(&self.path, &self.options)
    }

    fn expects_error(&self) -> bool {
        self.expects_error
    }
}
