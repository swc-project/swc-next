//! JavaScript and TypeScript APIs.
//!
//! The JavaScript facade resolves top-level settings and composes pipeline options.
//! Optional fields retain absence, and `Default` leaves them unset.

#[cfg(feature = "js_codegen")]
pub mod codegen;
#[cfg(feature = "js_minify")]
pub mod minify;
#[cfg(feature = "js_parser")]
pub mod parser;
#[cfg(feature = "js_transform")]
pub mod transform;

/// JavaScript and TypeScript stage settings.
///
/// Each stage field requires its corresponding `js_*` crate feature.
#[derive(Clone, Debug, Default)]
pub struct JsOptions {
    #[cfg(feature = "js_parser")]
    pub parser: Option<parser::JsParserOptions>,
    #[cfg(feature = "js_transform")]
    pub transform: Option<transform::JsTransformOptions>,
    /// A boolean shorthand or explicit compression and mangling settings.
    #[cfg(feature = "js_minify")]
    pub minify: Option<crate::BoolOr<minify::JsMinifyOptions>>,
    #[cfg(feature = "js_codegen")]
    pub codegen: Option<codegen::JsCodegenOptions>,
}
