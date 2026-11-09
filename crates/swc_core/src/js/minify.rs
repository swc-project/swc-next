use swc_next_ecma_minifier::{CompressOptions, MinifierOptions};

use crate::BoolOr;

/// JavaScript AST minification settings.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsMinifyOptions {
    /// Disable compression or enable it with defaults or explicit settings.
    pub compress: Option<BoolOr<JsCompressOptions>>,
    /// Enable local binding mangling. Additional mangling options are not yet
    /// implemented.
    pub mangle: Option<bool>,
    // Extend the boolean shorthand when mangler options are supported.
    // pub mangle: Option<BoolOr<JsMangleOptions>>,
}

impl From<JsMinifyOptions> for MinifierOptions {
    fn from(options: JsMinifyOptions) -> Self {
        let defaults = Self::default();
        Self {
            compress: match options.compress {
                None => defaults.compress,
                Some(BoolOr::Bool(false)) => None,
                Some(BoolOr::Bool(true)) => Some(CompressOptions::default()),
                Some(BoolOr::Options(options)) => Some(options.into()),
            },
            mangle: options.mangle.unwrap_or(defaults.mangle),
        }
    }
}

/// Implemented compression passes. Each pass defaults to enabled when
/// compression is enabled; unset fields preserve that decision for resolution.
#[derive(Clone, Copy, Debug, Default)]
pub struct JsCompressOptions {
    /// Remove empty statements and unnecessary blocks.
    pub blocks: Option<bool>,
    /// Shorten boolean literals.
    pub booleans: Option<bool>,
    /// Shorten comparisons whose operand types permit it.
    pub comparisons: Option<bool>,
    /// Shorten conditional statements and expressions.
    pub conditionals: Option<bool>,
    /// Remove code that a constant condition never runs.
    pub dead_code: Option<bool>,
    /// Fold constant expressions when the result is shorter.
    pub evaluate: Option<bool>,
    /// Shorten unshadowed global constants.
    pub global_defs: Option<bool>,
    /// Shorten conditional returns and remove redundant trailing returns.
    pub if_return: Option<bool>,
    /// Join adjacent declarations of the same kind.
    pub join_vars: Option<bool>,
    /// Negate immediately invoked function expressions in statement position.
    pub negate_iife: Option<bool>,
    /// Inline safe local constants when the replacement is shorter.
    pub reduce_vars: Option<bool>,
    /// Join expression statements with comma operators.
    pub sequences: Option<bool>,
    /// Remove unused computations that have no side effects.
    pub side_effects: Option<bool>,
    /// Shorten undefined `typeof` comparisons for local `var` bindings.
    pub typeofs: Option<bool>,
    /// Remove unused variables whose initializers have no side effects.
    pub unused: Option<bool>,
}

impl From<JsCompressOptions> for CompressOptions {
    fn from(options: JsCompressOptions) -> Self {
        let defaults = Self::default();
        Self {
            blocks: options.blocks.unwrap_or(defaults.blocks),
            booleans: options.booleans.unwrap_or(defaults.booleans),
            comparisons: options.comparisons.unwrap_or(defaults.comparisons),
            conditionals: options.conditionals.unwrap_or(defaults.conditionals),
            dead_code: options.dead_code.unwrap_or(defaults.dead_code),
            evaluate: options.evaluate.unwrap_or(defaults.evaluate),
            global_defs: options.global_defs.unwrap_or(defaults.global_defs),
            if_return: options.if_return.unwrap_or(defaults.if_return),
            join_vars: options.join_vars.unwrap_or(defaults.join_vars),
            negate_iife: options.negate_iife.unwrap_or(defaults.negate_iife),
            reduce_vars: options.reduce_vars.unwrap_or(defaults.reduce_vars),
            sequences: options.sequences.unwrap_or(defaults.sequences),
            side_effects: options.side_effects.unwrap_or(defaults.side_effects),
            typeofs: options.typeofs.unwrap_or(defaults.typeofs),
            unused: options.unused.unwrap_or(defaults.unused),
        }
    }
}

// pub struct JsMangleOptions {
//     pub top_level: Option<bool>,
//     pub keep_class_names: Option<bool>,
//     pub keep_fn_names: Option<bool>,
// }
