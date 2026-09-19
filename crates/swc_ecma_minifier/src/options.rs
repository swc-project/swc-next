use serde::{Deserialize, Serialize};

/// AST optimization options. These do not control code generation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinifierOptions {
    /// Compression settings. `None` disables all compression.
    pub compress: Option<CompressOptions>,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self {
            compress: Some(CompressOptions::default()),
        }
    }
}

/// Settings for the compression visitor.
///
/// Individual optimizations will be introduced with their options.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompressOptions {}
