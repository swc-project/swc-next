//! JavaScript and TypeScript APIs.
//!
//! The JavaScript facade resolves top-level settings and composes pipeline options.
//! Optional fields retain absence, and `Default` leaves them unset.

use crate::BoolOr;

pub mod codegen;
pub mod minify;
pub mod parser;
pub mod transform;

pub use codegen::JsCodegenOptions;
pub use minify::{JsCompressOptions, JsMinifyOptions};
pub use parser::JsParserOptions;
pub use transform::{
    JsFeature, JsTransformOptions, JsxOptions, ReactRefreshOptions, TypeScriptOptions,
};

/// JavaScript and TypeScript stage settings.
#[derive(Clone, Debug, Default)]
pub struct JsOptions {
    pub parser: Option<JsParserOptions>,
    pub transform: Option<JsTransformOptions>,
    /// A boolean shorthand or explicit compression and mangling settings.
    pub minify: Option<BoolOr<JsMinifyOptions>>,
    pub codegen: Option<JsCodegenOptions>,
}
