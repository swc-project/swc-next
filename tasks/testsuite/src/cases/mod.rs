use std::{
    fs,
    path::{Path, PathBuf},
};

use jwalk::WalkDir;
use serde::de::DeserializeOwned;
use swc_next_ecma_ast::{Lang, SourceType, Span};
use swc_next_ecma_codegen::Options as CodegenOptions;
use swc_next_ecma_transformer::TransformerOptions;

use crate::util::crate_root;

pub mod babel;
pub mod babel_parser;
pub mod swc_resolver;
pub mod swc_transform;
pub mod test262;
pub mod typescript;

pub trait Case: Sized + Sync {
    const NAME: &'static str;

    fn read() -> Vec<Self>;
    fn path(&self) -> &Path;
    fn name(&self) -> &str;
    fn code(&self) -> &str;
    fn source_type(&self) -> SourceType;
    fn expected_source_type(&self) -> Option<SourceType> {
        None
    }
    fn lang(&self) -> Lang;
    fn expects_error(&self) -> bool;
    /// Reference ranges that must contain every reported error span.
    /// Without ranges, negative cases only require an error.
    fn expected_error_spans(&self) -> Option<&[Span]> {
        None
    }
    /// Whether a case expected to succeed also asserts that semantic analysis
    /// produces no early-error diagnostics.
    fn asserts_semantic_success(&self) -> bool {
        true
    }

    fn relative_path(&self) -> &Path {
        self.path().strip_prefix(crate_root()).unwrap()
    }
}

/// Fixture adapters provide the configuration consumed by the shared transform pipeline.
pub trait TransformCase: Case {
    fn transformer_options(&self) -> TransformerOptions;

    /// Identifies the input unit when a fixture contains multiple virtual files.
    fn virtual_file_name(&self) -> Option<&str> {
        None
    }

    fn options_label(&self) -> Option<&str> {
        None
    }

    fn supports_options(&self) -> bool {
        true
    }

    /// An absent oracle requests JavaScript output validation and transform idempotence.
    fn expected_output(&self) -> Option<&str> {
        None
    }

    fn codegen_options(&self) -> CodegenOptions {
        CodegenOptions::default()
    }
}

pub fn fixtures() -> PathBuf {
    crate_root().join("fixtures")
}

fn swc_conformance() -> PathBuf {
    std::env::var_os("SWC_CONFORMANCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| fixtures().join("swc-conformance"))
}

fn fixture_json() -> PathBuf {
    swc_conformance().join("fixtures-json")
}

fn json_paths(suite: &str) -> Vec<PathBuf> {
    let root = fixture_json().join(suite);
    let mut paths = Vec::new();
    for entry in WalkDir::new(&root) {
        let entry = entry.unwrap_or_else(|error| panic!("{}: {error}", root.display()));
        let path = entry.path();
        if entry.file_type().is_file()
            && path
                .extension()
                .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    paths.sort_unstable();
    paths
}

fn read_json<T: DeserializeOwned>(path: &Path) -> T {
    let bytes = fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_slice(&bytes).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}
