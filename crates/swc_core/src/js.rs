//! JavaScript and TypeScript APIs.
//!
//! The JavaScript facade resolves top-level settings and composes pipeline options.
//! Optional fields retain absence, and `Default` leaves them unset.

use crate::BoolOr;

#[cfg(feature = "codegen")]
pub mod codegen;
pub mod minify;
#[cfg(feature = "parser")]
pub mod parser;
pub mod transform;

/// JavaScript and TypeScript stage settings.
///
/// Parser and code generation fields require their corresponding crate features.
#[derive(Clone, Debug, Default)]
pub struct JsOptions {
    #[cfg(feature = "parser")]
    pub parser: Option<parser::JsParserOptions>,
    pub transform: Option<transform::JsTransformOptions>,
    /// A boolean shorthand or explicit compression and mangling settings.
    pub minify: Option<BoolOr<minify::JsMinifyOptions>>,
    #[cfg(feature = "codegen")]
    pub codegen: Option<codegen::JsCodegenOptions>,
}
