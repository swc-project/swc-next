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
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompressOptions {
    /// Shorten boolean expressions: `true` to `!0` and `false` to `!1`.
    /// Enabled by default.
    pub booleans: bool,
    /// Shorten unshadowed global constants. Enabled by default.
    pub global_defs: bool,
    /// Shorten undefined typeof comparisons for local var bindings. Enabled by default.
    pub typeofs: bool,
    /// Remove unused local variables with inert initializers. Enabled by default.
    pub unused: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            booleans: true,
            global_defs: true,
            typeofs: true,
            unused: true,
        }
    }
}
