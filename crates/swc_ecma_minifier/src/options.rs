use serde::{Deserialize, Serialize};

/// AST optimization options. These do not control code generation.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MinifierOptions {
    /// Compression settings. `None` disables all compression.
    pub compress: Option<CompressOptions>,
    /// Shorten the names of local bindings. Top-level names are kept, and a
    /// program that uses `eval` or `with` is not mangled. Enabled by default.
    pub mangle: bool,
}

impl Default for MinifierOptions {
    fn default() -> Self {
        Self {
            compress: Some(CompressOptions::default()),
            mangle: true,
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
    /// Remove code that a constant condition never runs, such as the
    /// branches of `if (0) a(); else b()` and `!0 ? a : b`, `!1 && a`, and
    /// `while (0) a()`. `var` names declared in removed code are kept.
    /// Enabled by default.
    pub dead_code: bool,
    /// Fold operators applied to constants, such as `1 + 2` to `3` and
    /// `"a" + "b"` to `"ab"`, when the result prints shorter. Enabled by
    /// default.
    pub evaluate: bool,
    /// Shorten unshadowed global constants. Enabled by default.
    pub global_defs: bool,
    /// Shorten undefined typeof comparisons for local var bindings. Enabled by default.
    pub typeofs: bool,
    /// Remove unused local variables whose initializers have no side effects.
    /// Enabled by default.
    pub unused: bool,
}

impl Default for CompressOptions {
    fn default() -> Self {
        Self {
            booleans: true,
            dead_code: true,
            evaluate: true,
            global_defs: true,
            typeofs: true,
            unused: true,
        }
    }
}

impl CompressOptions {
    /// Settings that compress nothing, for a traversal that only mangles.
    pub(crate) fn disabled() -> Self {
        Self {
            booleans: false,
            dead_code: false,
            evaluate: false,
            global_defs: false,
            typeofs: false,
            unused: false,
        }
    }
}
