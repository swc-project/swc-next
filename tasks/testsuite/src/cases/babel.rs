use std::{
    collections::HashMap,
    fs::{self, read_to_string},
    io::{self, ErrorKind},
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use swc_experimental_ecma_ast::SourceType;

use crate::cases::fixtures;

pub mod options;

use options::BabelOptions;

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
        let mut options_cache = HashMap::<PathBuf, BabelOptions>::new();

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
                if !entry.file_type().is_file() || entry.file_name() != "input.js" {
                    continue;
                }

                let input_path = entry.path();
                let Some(case_dir) = input_path.parent() else {
                    continue;
                };
                let output_path = case_dir.join("output.js");
                if !output_path.is_file() {
                    continue;
                }

                let options_path = nearest_options(case_dir, &fixture_dir);
                let options = if let Some(path) = &options_path {
                    if let Some(options) = options_cache.get(path) {
                        options.clone()
                    } else {
                        let source = read_to_string(path)?;
                        let options =
                            serde_json::from_str::<BabelOptions>(&source).map_err(|error| {
                                io::Error::new(
                                    ErrorKind::InvalidData,
                                    format!("{}: {error}", path.display()),
                                )
                            })?;
                        options_cache.insert(path.clone(), options.clone());
                        options
                    }
                } else {
                    BabelOptions::default()
                };

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
        match self.options.source_type.as_deref() {
            Some("script" | "commonjs") => SourceType::Script,
            Some("module" | "unambiguous") | None => SourceType::Module,
            Some(_) => SourceType::Module,
        }
    }
}

fn nearest_options(case_dir: &Path, fixture_dir: &Path) -> Option<PathBuf> {
    let mut directory = case_dir;
    loop {
        let candidate = directory.join("options.json");
        if candidate.is_file() {
            return Some(candidate);
        }
        if directory == fixture_dir {
            return None;
        }
        directory = directory.parent()?;
        if !directory.starts_with(fixture_dir) {
            return None;
        }
    }
}
