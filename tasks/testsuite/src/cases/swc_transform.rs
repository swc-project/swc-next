use std::{
    fs, io,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use swc_next_ecma_ast::{Lang, SourceType};
use swc_next_ecma_transformer::{TransformerOptions, es2022::ES2022Options};

use super::{Case, fixtures, transform::TransformCase};

#[derive(Debug)]
pub struct SwcTransformCase {
    path: PathBuf,
    output_path: PathBuf,
    name: String,
    input: String,
    output: String,
    pub requires_class_properties: bool,
}

impl SwcTransformCase {
    fn try_read() -> io::Result<Vec<Self>> {
        let fixture_dir =
            fixtures().join("swc/crates/swc_ecma_transforms_compat/tests/static-blocks");
        let mut cases = Vec::new();

        for entry in WalkDir::new(&fixture_dir) {
            let entry = entry.map_err(io::Error::other)?;
            if !entry.file_type().is_file() || entry.file_name() != "input.js" {
                continue;
            }

            let path = entry.path();
            let case_dir = path.parent().unwrap();
            let relative_dir = case_dir.strip_prefix(&fixture_dir).unwrap();
            let output_path = case_dir.join("output.js");
            let requires_class_properties = relative_dir.starts_with("class-properties");
            cases.push(Self {
                name: format!(
                    "static-blocks/{}",
                    relative_dir.to_string_lossy().replace('\\', "/")
                ),
                input: fs::read_to_string(&path)?,
                output: fs::read_to_string(&output_path)?,
                path,
                output_path,
                requires_class_properties,
            });
        }

        cases.sort_unstable_by(|left, right| left.path.cmp(&right.path));
        Ok(cases)
    }
}

impl Case for SwcTransformCase {
    const NAME: &'static str = "swc";

    fn read() -> Vec<Self> {
        Self::try_read()
            .expect("failed to read SWC transform fixtures; run ./scripts/clone_fixtures.sh")
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
        SourceType::Module
    }

    fn lang(&self) -> Lang {
        Lang::Js
    }

    fn expects_error(&self) -> bool {
        false
    }
}

impl TransformCase for SwcTransformCase {
    fn package_name(&self) -> &str {
        "static-blocks"
    }

    fn output_path(&self) -> &Path {
        &self.output_path
    }

    fn output(&self) -> &str {
        &self.output
    }

    fn transformer_options(&self) -> TransformerOptions {
        TransformerOptions {
            es2022: ES2022Options {
                static_blocks: true,
            },
            ..TransformerOptions::default()
        }
    }
}
