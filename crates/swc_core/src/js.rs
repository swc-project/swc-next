//! JavaScript and TypeScript APIs.

use swc_next_ecma_ast::Diagnostic;

#[cfg(feature = "js_codegen")]
pub mod codegen;
#[cfg(feature = "js_transform")]
pub mod minify;
#[cfg(feature = "js_parser")]
pub mod parser;
#[cfg(feature = "js_transform")]
pub mod transform;

/// JavaScript and TypeScript stage settings.
///
/// Each stage field requires its corresponding `js_*` crate feature.
/// Minification settings are included with `js_transform`.
#[derive(Clone, Debug, Default)]
pub struct JsOptions {
    /// Used to infer `lang` and `js.parser.sourceType`, and
    /// also for some transformation, diagnostics and source maps.
    pub filename: Option<String>,
    /// Generate a separate source map in `JsOutput.map`. Defaults to false.
    /// Uses `filename` as the source path, or `input.js` when it is absent.
    pub source_maps: Option<bool>,
    // Override the shared target for all JavaScript stages.
    // pub target: Option<Target>,
    /// Comment handling policy for the pipeline. Defaults to `All` without
    /// minification and `None` with minification until `Some` is supported.
    pub comments: Option<CommentsOption>,
    #[cfg(feature = "js_parser")]
    pub parser: Option<parser::JsParserOptions>,
    #[cfg(feature = "js_transform")]
    pub transform: Option<transform::JsTransformOptions>,
    /// A boolean shorthand or explicit compression and mangling settings.
    #[cfg(feature = "js_transform")]
    pub minify: Option<crate::BoolOr<minify::JsMinifyOptions>>,
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

/// Comment policies currently supported by the JavaScript pipeline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CommentsOption {
    /// Keep all comments inline, including legal comments and annotations.
    All,
    // Some,
    /// Remove all comments from the output. Transform pragmas are still read.
    None,
    // Options(CommentsOptions),
}

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

/// Rust pipeline output. Errors stop the pipeline and leave `code` empty.
#[derive(Debug, Default)]
pub struct JsOutput {
    pub code: String,
    /// Source Map v3 JSON when `source_maps` is enabled and the pipeline succeeds.
    pub map: Option<String>,
    /// Legal comments when `comments.legal` is `"external"`.
    /// Currently always `None` until comment policies are supported.
    pub legal_comments: Option<Vec<String>>,
    /// Owned parser, semantic, and transform diagnostics with UTF-8 byte offsets.
    pub diagnostics: Vec<Diagnostic<'static>>,
}
