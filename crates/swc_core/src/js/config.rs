//! JavaScript and TypeScript stage configuration.
//!
//! The JavaScript facade resolves top-level settings and composes pipeline options.
//! Optional fields retain absence, and `Default` leaves them unset.

use crate::BoolOr;

mod codegen;
mod minify;
mod parser;
mod transform;

pub use codegen::JsCodegenOptions;
pub use minify::{JsCompressOptions, JsMinifyOptions};
pub use parser::JsParserOptions;
pub use swc_next_ecma_ast::{Lang, SourceType};
pub use swc_next_ecma_parser::CommentMode;
pub use swc_next_ecma_transformer::jsx::JsxRuntime;
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
