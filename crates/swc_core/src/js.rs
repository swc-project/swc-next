//! JavaScript and TypeScript APIs.

use swc_next_ecma_ast::Diagnostic;

#[cfg(all(feature = "bridge", feature = "js_parser"))]
use swc_next_bridge::bridge;

#[cfg(all(feature = "bridge", feature = "js_parser", not(target_family = "wasm")))]
use swc_next_bridge::napi;

/// Comment policy.
#[cfg_attr(
    all(feature = "bridge", feature = "js_parser"),
    bridge(string_enum = "lowercase", js_name = "CommentMode")
)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentsOption {
    /// Keep all comments.
    All,
    // Some,
    /// Remove all comments.
    None,
    // Options(CommentsOptions),
}

#[cfg(feature = "js_codegen")]
pub mod codegen;
#[cfg(feature = "js_transform")]
pub mod minify;
#[cfg(feature = "js_parser")]
pub mod parser;
#[cfg(feature = "js_transform")]
pub mod transform;

/// JavaScript and TypeScript options.
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(default, rename_all = "camelCase", deny_unknown_fields)
)]
#[derive(Clone, Debug, Default)]
pub struct JsOptions {
    /// Used to infer the language and source type, and for diagnostics and source maps.
    pub filename: Option<String>,
    /// Source map generation.
    ///
    /// @default false
    #[cfg_attr(feature = "serde", serde(rename = "sourceMap"))]
    pub source_maps: Option<bool>,
    // Override the shared target for all JavaScript stages.
    // pub target: Option<Target>,
    /// Comment policy.
    pub comments: Option<CommentsOption>,
    /// Parser options.
    #[cfg(feature = "js_parser")]
    pub parser: Option<parser::JsParserOptions>,
    /// Transform options.
    #[cfg(feature = "js_transform")]
    pub transform: Option<transform::JsTransformOptions>,
    /// Minification. `true` uses default options.
    #[cfg(feature = "js_transform")]
    pub minify: Option<crate::BoolOr<minify::JsMinifyOptions>>,
    /// Code generation options.
    #[cfg(feature = "js_codegen")]
    pub codegen: Option<codegen::JsCodegenOptions>,
}

// Draft-only shared types reachable from JsOptions. These declarations remain
// commented out until their policies are implemented.
// pub enum Target {
//     EsVersion(EsVersion),
//     Browserslist(Vec<String>),
//     Engines(BTreeMap<Engine, String>),
// }
//
// pub enum EsVersion {
//     Es5,
//     Es2015,
//     Es2016,
//     Es2017,
//     Es2018,
//     Es2019,
//     Es2020,
//     Es2021,
//     Es2022,
//     Es2023,
//     Es2024,
//     Es2025,
//     EsNext,
// }
//
// pub enum Engine {
//     Chrome,
//     Edge,
//     Firefox,
//     Safari,
//     Ios,
//     Opera,
//     Samsung,
//     Node,
//     Deno,
//     Bun,
// }

// pub struct CommentsOptions {
//     pub other: Option<OtherComments>,
//     pub legal: Option<LegalComments>,
//     pub annotations: Option<bool>, // JavaScript annotations; defaults to true.
//     pub keep: Option<Vec<String>>, // Regular-expression patterns.
// }
//
// pub enum OtherComments {
//     All,
//     None,
// }
//
// pub enum LegalComments {
//     Inline,
//     Eof,
//     External,
//     None,
// }

/// Source transformation output.
#[derive(Debug, Default)]
pub struct JsOutput {
    pub code: String,
    /// Source map as a JSON string. `None` when source maps are disabled.
    pub map: Option<String>,
    /// Legal comments when `comments.legal` is `"external"`.
    pub legal_comments: Option<Vec<String>>,
    /// Diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}
