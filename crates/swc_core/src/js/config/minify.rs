use crate::BoolOr;

/// JavaScript AST minification settings.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsMinifyOptions {
    /// Disable compression or enable it with defaults or explicit settings.
    pub compress: Option<BoolOr<JsCompressOptions>>,
    /// Enable local binding mangling. Additional mangling options are not yet
    /// implemented.
    pub mangle: Option<bool>,
}

/// Implemented compression passes. Each pass defaults to enabled when
/// compression is enabled; unset fields preserve that decision for resolution.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsCompressOptions {
    /// Remove empty statements and unnecessary blocks.
    pub blocks: Option<bool>,
    /// Shorten boolean literals.
    pub booleans: Option<bool>,
    /// Remove code that a constant condition never runs.
    pub dead_code: Option<bool>,
    /// Fold constant expressions when the result is shorter.
    pub evaluate: Option<bool>,
    /// Shorten unshadowed global constants.
    pub global_defs: Option<bool>,
    /// Join adjacent declarations of the same kind.
    pub join_vars: Option<bool>,
    /// Join expression statements with comma operators.
    pub sequences: Option<bool>,
    /// Shorten undefined `typeof` comparisons for local `var` bindings.
    pub typeofs: Option<bool>,
    /// Remove unused variables whose initializers have no side effects.
    pub unused: Option<bool>,
}
