use std::{
    fs, io,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use rayon::prelude::*;
use swc_core::base::config::Config;
use swc_next_core::js::JsTransformApiOptions;
use swc_next_ecma_ast::{Lang, SourceType};

use super::{Case, TransformCase, fixtures};
use crate::util::{read_optional, with_suffix};
mod options;

#[derive(Debug)]
pub struct SwcTransformCase {
    path: PathBuf,
    name: String,
    input: String,
    output: String,
    options: Option<Config>,
    unsupported_snapshot: bool,
}

impl SwcTransformCase {
    fn try_read() -> io::Result<Vec<Self>> {
        let root = fixtures().join("swc/crates/swc/tests/compile");
        let mut paths = Vec::new();
        for entry in WalkDir::new(&root) {
            let entry = entry.map_err(io::Error::other)?;
            if !entry.file_type().is_dir() || entry.file_name() != "input" {
                continue;
            }
            let input_dir = entry.path();
            // The fixture macro ignores hidden fixture directories, but the
            // compiler walks every source inside a selected input directory.
            for entry in WalkDir::new(&input_dir).skip_hidden(false) {
                let entry = entry.map_err(io::Error::other)?;
                let path = entry.path();
                if !entry.file_type().is_file()
                    || !matches!(
                        path.extension().and_then(|v| v.to_str()),
                        Some("js" | "jsx" | "ts" | "tsx")
                    )
                {
                    continue;
                }
                let relative_path = path.strip_prefix(&input_dir).unwrap();
                let output = input_dir
                    .parent()
                    .unwrap()
                    .join("output")
                    .join(relative_path);
                paths.push((path, output));
            }
        }
        let mut cases = paths
            .into_par_iter()
            .map(|(path, output)| {
                let options = options::from_test_path(&path, &root)?;
                // This suite compares emitted code. Diagnostics, source maps and
                // additional compiler products need APIs that are not yet exposed.
                let mut unsupported_snapshot = false;
                for suffix in [".stderr", ".map", ".d.ts", ".extra.json"] {
                    let snapshot = read_optional(&with_suffix(&output, suffix))?;
                    unsupported_snapshot |= snapshot.is_some_and(|source| !source.is_empty());
                }
                let raw = with_suffix(&output, ".raw");
                let output = if raw.is_file() { raw } else { output };
                let output = read_optional(&output)?.unwrap_or_default();
                let relative_path = path.strip_prefix(&root).unwrap();
                Ok(Self {
                    name: relative_path.to_string_lossy().replace('\\', "/"),
                    input: fs::read_to_string(&path)?,
                    path,
                    output,
                    options,
                    unsupported_snapshot,
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        cases.sort_unstable_by(|a, b| a.path.cmp(&b.path));
        Ok(cases)
    }
}

impl Case for SwcTransformCase {
    const NAME: &'static str = "swc";

    fn read() -> Vec<Self> {
        Self::try_read().unwrap()
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
        self.options
            .as_ref()
            .map_or(SourceType::Module, options::source_type)
    }

    fn lang(&self) -> Lang {
        self.options.as_ref().map_or(Lang::Js, options::lang)
    }

    fn expects_error(&self) -> bool {
        false
    }
}

impl TransformCase for SwcTransformCase {
    fn supports_options(&self) -> bool {
        !self.unsupported_snapshot && self.options.as_ref().is_some_and(options::supports_options)
    }

    fn expected_output(&self) -> Option<&str> {
        Some(&self.output)
    }

    fn options(&self) -> JsTransformApiOptions {
        options::options(self.options.as_ref().unwrap(), &self.path)
    }
}
